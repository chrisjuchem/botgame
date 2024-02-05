@echo off
For /f "tokens=2-4 delims=/ " %%a in ('date /t') do (set mydate=%%c-%%a-%%b)
For /f "tokens=1-3 delims=/:." %%a in ("%TIME%") do (set mytime=%%a-%%b-%%c)

Set RUST_BACKTRACE=1
.\client.exe assets/config1.json > .\logs\%mydate%_%mytime%.log 2>&1
