# Runner
Runner is a program to run multiple executables in parallel. I mostly use it to run a server program while also rebuilding a frontend in the background.
The configuration for it is through a kdl file in the directory it is invoked from. Here's an example config file:

```kdl
run "frontend" {
  path "frontend"
  command "npm"
  args "run" "dev" 
}

run "backend" {
  path "backend"
  command "cargo"
  args "run" "--color=always"
}
```

This builds a frontend through npm and a cargo server to actually run the backend server. 
It takes paths to run the commands in as their working directory, arguments are optional if not needed.