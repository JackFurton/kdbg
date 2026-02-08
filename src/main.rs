use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use serde_json::Value;
use std::process::{Command, Stdio};

#[derive(Parser)]
#[command(name = "kdbg")]
#[command(about = "Kubernetes Pod Debugger - Fast kubectl wrapper", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List all pods
    List {
        /// Namespace (default: all)
        #[arg(short, long)]
        namespace: Option<String>,
        
        /// Show more details
        #[arg(short, long)]
        verbose: bool,
    },
    
    /// Get pod logs
    Logs {
        /// Pod name (or partial match)
        pod: String,
        
        /// Namespace
        #[arg(short, long)]
        namespace: Option<String>,
        
        /// Follow logs
        #[arg(short, long)]
        follow: bool,
        
        /// Number of lines
        #[arg(long, default_value = "100")]
        tail: u32,
    },
    
    /// Execute command in pod
    Exec {
        /// Pod name (or partial match)
        pod: String,
        
        /// Namespace
        #[arg(short, long)]
        namespace: Option<String>,
        
        /// Command to run (default: /bin/sh)
        #[arg(short, long, default_value = "/bin/sh")]
        command: String,
    },
    
    /// Describe pod
    Describe {
        /// Pod name (or partial match)
        pod: String,
        
        /// Namespace
        #[arg(short, long)]
        namespace: Option<String>,
    },
    
    /// Show pod resource usage
    Top {
        /// Namespace
        #[arg(short, long)]
        namespace: Option<String>,
    },
    
    /// Port forward to pod
    Forward {
        /// Pod name (or partial match)
        pod: String,
        
        /// Local port
        local_port: u16,
        
        /// Pod port
        pod_port: u16,
        
        /// Namespace
        #[arg(short, long)]
        namespace: Option<String>,
    },
    
    /// Open interactive shell in pod
    Shell {
        /// Pod name (or partial match)
        pod: String,
        
        /// Namespace
        #[arg(short, long)]
        namespace: Option<String>,
    },
    
    /// Create debug pod and shell into it
    Debug {
        /// Container image (default: busybox)
        #[arg(short, long, default_value = "busybox")]
        image: String,
        
        /// Namespace
        #[arg(short, long, default_value = "default")]
        namespace: String,
    },
    
    /// Restart pod (delete and let it recreate)
    Restart {
        /// Pod name (or partial match)
        pod: String,
        
        /// Namespace
        #[arg(short, long)]
        namespace: Option<String>,
    },
    
    /// Show pod events
    Events {
        /// Pod name (or partial match)
        pod: String,
        
        /// Namespace
        #[arg(short, long)]
        namespace: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::List { namespace, verbose } => list_pods(namespace, verbose)?,
        Commands::Logs { pod, namespace, follow, tail } => show_logs(&pod, namespace, follow, tail)?,
        Commands::Exec { pod, namespace, command } => exec_pod(&pod, namespace, &command)?,
        Commands::Describe { pod, namespace } => describe_pod(&pod, namespace)?,
        Commands::Top { namespace } => show_top(namespace)?,
        Commands::Forward { pod, local_port, pod_port, namespace } => {
            port_forward(&pod, local_port, pod_port, namespace)?
        }
        Commands::Shell { pod, namespace } => shell_pod(&pod, namespace)?,
        Commands::Debug { image, namespace } => debug_pod(&image, &namespace)?,
        Commands::Restart { pod, namespace } => restart_pod(&pod, namespace)?,
        Commands::Events { pod, namespace } => show_events(&pod, namespace)?,
    }
    
    Ok(())
}

fn list_pods(namespace: Option<String>, verbose: bool) -> Result<()> {
    let mut args = vec!["get", "pods"];
    
    let ns_str;
    if let Some(ns) = &namespace {
        ns_str = ns.clone();
        args.extend(&["-n", &ns_str]);
    } else {
        args.push("--all-namespaces");
    }
    
    args.push("-o");
    args.push("json");
    
    let output = Command::new("kubectl")
        .args(&args)
        .output()?;
    
    if !output.status.success() {
        eprintln!("{} kubectl command failed", "[ERROR]".red());
        return Ok(());
    }
    
    let json: Value = serde_json::from_slice(&output.stdout)?;
    let empty_vec = vec![];
    let pods = json["items"].as_array().unwrap_or(&empty_vec);
    
    println!("{}", "Pods:".cyan().bold());
    println!("{}", "-".repeat(100));
    
    if verbose {
        println!("{:<40} {:<15} {:<10} {:<15} {:<20}", 
            "NAME", "NAMESPACE", "STATUS", "RESTARTS", "AGE");
        println!("{}", "-".repeat(100));
    } else {
        println!("{:<40} {:<15} {:<10}", "NAME", "NAMESPACE", "STATUS");
        println!("{}", "-".repeat(100));
    }
    
    for pod in pods {
        let name = pod["metadata"]["name"].as_str().unwrap_or("unknown");
        let ns = pod["metadata"]["namespace"].as_str().unwrap_or("default");
        let phase = pod["status"]["phase"].as_str().unwrap_or("Unknown");
        
        let status_colored = match phase {
            "Running" => phase.green(),
            "Pending" => phase.yellow(),
            "Failed" => phase.red(),
            "Succeeded" => phase.blue(),
            _ => phase.normal(),
        };
        
        if verbose {
            let restarts = pod["status"]["containerStatuses"]
                .as_array()
                .and_then(|cs| cs.first())
                .and_then(|c| c["restartCount"].as_u64())
                .unwrap_or(0);
            
            let age = pod["metadata"]["creationTimestamp"]
                .as_str()
                .map(|ts| calculate_age(ts))
                .unwrap_or("unknown".to_string());
            
            println!("{:<40} {:<15} {:<10} {:<15} {:<20}", 
                name.cyan(), ns.bright_black(), status_colored, restarts, age);
        } else {
            println!("{:<40} {:<15} {:<10}", name.cyan(), ns.bright_black(), status_colored);
        }
    }
    
    println!("\nTotal: {} pods", pods.len());
    
    Ok(())
}

fn find_pod(pod_pattern: &str, namespace: Option<String>) -> Result<(String, String)> {
    let mut args = vec!["get", "pods"];
    
    let ns_str;
    if let Some(ns) = &namespace {
        ns_str = ns.clone();
        args.extend(&["-n", &ns_str]);
    } else {
        args.push("--all-namespaces");
    }
    
    args.extend(&["-o", "json"]);
    
    let output = Command::new("kubectl")
        .args(&args)
        .output()?;
    
    let json: Value = serde_json::from_slice(&output.stdout)?;
    let empty_vec = vec![];
    let pods = json["items"].as_array().unwrap_or(&empty_vec);
    
    let matches: Vec<_> = pods.iter()
        .filter(|pod| {
            let name = pod["metadata"]["name"].as_str().unwrap_or("");
            name.contains(pod_pattern)
        })
        .collect();
    
    if matches.is_empty() {
        anyhow::bail!("No pods found matching '{}'", pod_pattern);
    }
    
    if matches.len() > 1 {
        println!("{} Multiple pods found:", "[INFO]".yellow());
        for pod in &matches {
            let name = pod["metadata"]["name"].as_str().unwrap_or("unknown");
            let ns = pod["metadata"]["namespace"].as_str().unwrap_or("default");
            println!("  - {} (namespace: {})", name.cyan(), ns.bright_black());
        }
        anyhow::bail!("Please be more specific");
    }
    
    let pod = matches[0];
    let name = pod["metadata"]["name"].as_str().unwrap_or("unknown").to_string();
    let ns = pod["metadata"]["namespace"].as_str().unwrap_or("default").to_string();
    
    Ok((name, ns))
}

fn show_logs(pod_pattern: &str, namespace: Option<String>, follow: bool, tail: u32) -> Result<()> {
    let (pod_name, ns) = find_pod(pod_pattern, namespace)?;
    
    println!("{} Logs for pod: {} (namespace: {})", 
        "[INFO]".cyan(), pod_name.bold(), ns.bright_black());
    println!("{}", "-".repeat(100));
    
    let tail_str = tail.to_string();
    let mut args = vec!["logs", &pod_name, "-n", &ns, "--tail", &tail_str];
    
    if follow {
        args.push("-f");
    }
    
    let status = Command::new("kubectl")
        .args(&args)
        .status()?;
    
    if !status.success() {
        anyhow::bail!("Failed to get logs");
    }
    
    Ok(())
}

fn exec_pod(pod_pattern: &str, namespace: Option<String>, command: &str) -> Result<()> {
    let (pod_name, ns) = find_pod(pod_pattern, namespace)?;
    
    println!("{} Executing in pod: {} (namespace: {})", 
        "[INFO]".cyan(), pod_name.bold(), ns.bright_black());
    println!("{} Command: {}", "[INFO]".cyan(), command.yellow());
    println!("{}", "-".repeat(100));
    
    let status = Command::new("kubectl")
        .args(&["exec", "-it", &pod_name, "-n", &ns, "--", command])
        .status()?;
    
    if !status.success() {
        anyhow::bail!("Failed to exec into pod");
    }
    
    Ok(())
}

fn describe_pod(pod_pattern: &str, namespace: Option<String>) -> Result<()> {
    let (pod_name, ns) = find_pod(pod_pattern, namespace)?;
    
    println!("{} Describing pod: {} (namespace: {})", 
        "[INFO]".cyan(), pod_name.bold(), ns.bright_black());
    println!("{}", "-".repeat(100));
    
    let status = Command::new("kubectl")
        .args(&["describe", "pod", &pod_name, "-n", &ns])
        .status()?;
    
    if !status.success() {
        anyhow::bail!("Failed to describe pod");
    }
    
    Ok(())
}

fn show_top(namespace: Option<String>) -> Result<()> {
    let mut args = vec!["top", "pods"];
    
    let ns_str;
    if let Some(ns) = &namespace {
        ns_str = ns.clone();
        args.extend(&["-n", &ns_str]);
    } else {
        args.push("--all-namespaces");
    }
    
    println!("{}", "Pod Resource Usage:".cyan().bold());
    println!("{}", "-".repeat(100));
    
    let status = Command::new("kubectl")
        .args(&args)
        .status()?;
    
    if !status.success() {
        eprintln!("{} Failed to get resource usage (metrics-server may not be installed)", 
            "[WARN]".yellow());
    }
    
    Ok(())
}

fn port_forward(pod_pattern: &str, local_port: u16, pod_port: u16, namespace: Option<String>) -> Result<()> {
    let (pod_name, ns) = find_pod(pod_pattern, namespace)?;
    
    println!("{} Port forwarding: localhost:{} -> {}:{} (namespace: {})", 
        "[INFO]".cyan(), local_port, pod_name.bold(), pod_port, ns.bright_black());
    println!("{} Press Ctrl+C to stop", "[INFO]".yellow());
    println!("{}", "-".repeat(100));
    
    let status = Command::new("kubectl")
        .args(&[
            "port-forward",
            &pod_name,
            &format!("{}:{}", local_port, pod_port),
            "-n",
            &ns,
        ])
        .status()?;
    
    if !status.success() {
        anyhow::bail!("Port forwarding failed");
    }
    
    Ok(())
}

fn shell_pod(pod_pattern: &str, namespace: Option<String>) -> Result<()> {
    let (pod_name, ns) = find_pod(pod_pattern, namespace)?;
    
    println!("{} Opening shell in pod: {} (namespace: {})", 
        "[INFO]".cyan(), pod_name.bold(), ns.bright_black());
    println!("{}", "-".repeat(100));
    
    // Try bash first, fall back to sh
    let shells = ["/bin/bash", "/bin/sh"];
    
    for (i, shell) in shells.iter().enumerate() {
        let mut cmd = Command::new("kubectl");
        cmd.args(&["exec", "-it", &pod_name, "-n", &ns, "--", shell]);
        
        // Inherit stdin/stdout/stderr for interactive shell
        cmd.stdin(Stdio::inherit())
           .stdout(Stdio::inherit())
           .stderr(Stdio::null()); // Suppress error messages when trying shells
        
        let status = cmd.status()?;
        
        if status.success() {
            return Ok(());
        }
        
        // If bash failed, try sh (last attempt with stderr visible)
        if i == shells.len() - 1 {
            let mut cmd = Command::new("kubectl");
            cmd.args(&["exec", "-it", &pod_name, "-n", &ns, "--", shell]);
            cmd.stdin(Stdio::inherit())
               .stdout(Stdio::inherit())
               .stderr(Stdio::inherit());
            
            let status = cmd.status()?;
            if status.success() {
                return Ok(());
            }
        }
    }
    
    anyhow::bail!("Failed to open shell (tried bash and sh)")
}

fn debug_pod(image: &str, namespace: &str) -> Result<()> {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let pod_name = format!("debug-{}", timestamp);
    
    println!("{} Creating debug pod: {} (image: {}, namespace: {})", 
        "[INFO]".cyan(), pod_name.bold(), image.yellow(), namespace.bright_black());
    println!("{} Pod will be deleted when you exit the shell", "[INFO]".yellow());
    println!("{}", "-".repeat(100));
    
    // Create pod
    let output = Command::new("kubectl")
        .args(&[
            "run",
            &pod_name,
            "--image", image,
            "-n", namespace,
            "--restart=Never",
            "--rm",
            "-it",
            "--",
            "/bin/sh",
        ])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    
    if !output.success() {
        anyhow::bail!("Failed to create debug pod");
    }
    
    Ok(())
}

fn restart_pod(pod_pattern: &str, namespace: Option<String>) -> Result<()> {
    let (pod_name, ns) = find_pod(pod_pattern, namespace)?;
    
    println!("{} Restarting pod: {} (namespace: {})", 
        "[INFO]".cyan(), pod_name.bold(), ns.bright_black());
    println!("{} This will delete the pod and let the controller recreate it", 
        "[INFO]".yellow());
    println!("{}", "-".repeat(100));
    
    let status = Command::new("kubectl")
        .args(&["delete", "pod", &pod_name, "-n", &ns])
        .status()?;
    
    if !status.success() {
        anyhow::bail!("Failed to delete pod");
    }
    
    println!("{} Pod deleted. Waiting for recreation...", "[SUCCESS]".green());
    
    Ok(())
}

fn show_events(pod_pattern: &str, namespace: Option<String>) -> Result<()> {
    let (pod_name, ns) = find_pod(pod_pattern, namespace)?;
    
    println!("{} Events for pod: {} (namespace: {})", 
        "[INFO]".cyan(), pod_name.bold(), ns.bright_black());
    println!("{}", "-".repeat(100));
    
    let status = Command::new("kubectl")
        .args(&[
            "get", "events",
            "-n", &ns,
            "--field-selector", &format!("involvedObject.name={}", pod_name),
            "--sort-by", ".lastTimestamp",
        ])
        .status()?;
    
    if !status.success() {
        anyhow::bail!("Failed to get events");
    }
    
    Ok(())
}

fn calculate_age(timestamp: &str) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    let created = chrono::DateTime::parse_from_rfc3339(timestamp)
        .map(|dt| dt.timestamp())
        .unwrap_or(0);
    
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    
    let diff = now - created;
    
    if diff < 60 {
        format!("{}s", diff)
    } else if diff < 3600 {
        format!("{}m", diff / 60)
    } else if diff < 86400 {
        format!("{}h", diff / 3600)
    } else {
        format!("{}d", diff / 86400)
    }
}
