# kdbg - Kubernetes Pod Debugger

Fast kubectl wrapper for common debugging tasks. No more typing long kubectl commands.

## Features

- List pods with fuzzy matching
- Get logs with auto-completion
- Exec into pods by partial name
- Port forwarding
- Resource usage monitoring
- Describe pods

## Installation

```bash
cargo build --release
sudo cp target/release/kdbg /usr/local/bin/
```

## Usage

### List all pods
```bash
kdbg list
kdbg list -n my-namespace
kdbg list -v  # verbose mode with age and restarts
```

### Get logs
```bash
kdbg logs my-pod
kdbg logs my-pod -f  # follow logs
kdbg logs my-pod --tail 50
kdbg logs my-pod -n my-namespace
```

### Execute command in pod
```bash
kdbg exec my-pod  # opens /bin/sh
kdbg exec my-pod -c /bin/bash
kdbg exec my-pod -c "ls -la /app"
```

### Open interactive shell
```bash
kdbg shell my-pod  # auto-detects bash or sh
kdbg shell my-pod -n my-namespace
```

### Create debug pod
```bash
kdbg debug                    # Creates busybox pod and shells into it
kdbg debug --image ubuntu     # Creates ubuntu debug pod
kdbg debug --image nicolaka/netshoot  # Network debugging tools
```

The debug pod is automatically deleted when you exit the shell.

### Describe pod
```bash
kdbg describe my-pod
kdbg describe my-pod -n my-namespace
```

### Show resource usage
```bash
kdbg top
kdbg top -n my-namespace
```

### Port forward
```bash
kdbg forward my-pod 8080 80  # localhost:8080 -> pod:80
kdbg forward my-pod 3000 3000 -n my-namespace
```

### Restart pod
```bash
kdbg restart my-pod  # Deletes pod, lets deployment recreate it
kdbg restart my-pod -n production
```

### Show pod events
```bash
kdbg events my-pod  # Shows recent events for debugging
kdbg events my-pod -n my-namespace
```

## Fuzzy Matching

All commands support partial pod names:

```bash
# Instead of typing the full pod name:
kubectl logs my-app-deployment-7d4f8c9b5-xk2lp

# Just use part of it:
kdbg logs my-app
```

If multiple pods match, kdbg will show you the options.

## Why kdbg?

**Before:**
```bash
kubectl get pods --all-namespaces | grep my-app
kubectl logs my-app-deployment-7d4f8c9b5-xk2lp -n production --tail 100 -f
kubectl exec -it my-app-deployment-7d4f8c9b5-xk2lp -n production -- /bin/sh
```

**After:**
```bash
kdbg list
kdbg logs my-app -f
kdbg exec my-app
```

## Requirements

- kubectl installed and configured
- Rust 1.70+ (for building)

## License

MIT
