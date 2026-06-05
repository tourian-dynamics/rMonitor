# Support Guide for `rMonitor`

Thank you for using `rMonitor`! If you run into issues, need assistance, or want to ask questions, please refer to this guide.

## 🩺 Step 1: Run Diagnostics
Before reporting a bug, please run the built-in diagnostic tool to check if the issue is environment-related (e.g., missing registry keys, restricted terminal size, or network adapter permissions):
```powershell
.\rmon.exe --doctor
```
This will print an audit report including:
*   Execution privilege level (Standard vs Elevated)
*   Registry permissions for CPU theme configs and GPU descriptors
*   Detected graphics controllers and interface connection statuses
*   Log file path

## 📝 Step 2: Check the Logs
`rMonitor` logs execution steps, interface swaps, registry access issues, and system panic backtraces silently to:
`%APPDATA%\rmonitor\rmonitor.log`

Please check this log file for any error entries around the time the issue occurred.

## 🐛 Step 3: File an Issue
If diagnostics and logs show that the application is misbehaving, please file a bug report on the repository issues tab. Make sure to include:
1. The exact version of Windows you are running.
2. The output of the `.\rmon.exe --doctor` command.
3. The matching log lines from your `%APPDATA%\rmonitor\rmonitor.log` file.
4. Steps to reproduce the bug.

## 💡 Feature Requests
We welcome feature requests and design enhancements! Please open a feature request issue and clearly describe:
*   The problem you are trying to solve.
*   Your proposed visual layout or functional enhancement.
*   Any hardware configuration considerations (e.g., multi-GPU, high core counts).
