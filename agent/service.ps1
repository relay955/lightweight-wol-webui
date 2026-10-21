#Requires -RunAsAdministrator
[CmdletBinding()]
param(
    [ValidateSet('Install', 'Uninstall')]
    [string]$Action = 'Install',
    [string]$ExecutablePath = (Join-Path $PSScriptRoot 'lightweight-wol-agent.exe')
)

$ErrorActionPreference = 'Stop'
$serviceName = 'LightweightWolAgent'
$service = Get-Service -Name $serviceName -ErrorAction SilentlyContinue

if ($Action -eq 'Uninstall') {
    if ($null -eq $service) {
        Write-Host 'Service is not installed.'
        return
    }
    Stop-Service -Name $serviceName
    $service.WaitForStatus('Stopped', [TimeSpan]::FromSeconds(15))
    & sc.exe delete $serviceName
    if ($LASTEXITCODE -ne 0) { throw 'Failed to delete the service.' }
    Write-Host 'Service removed. Executable, configuration and logs were preserved.'
    return
}

if ($null -ne $service) { throw 'Service already exists. Uninstall it before reinstalling.' }
$executable = (Resolve-Path -LiteralPath $ExecutablePath).ProviderPath
if (-not (Test-Path -LiteralPath $executable -PathType Leaf)) { throw 'Executable must be a file.' }
$configPath = Join-Path (Split-Path -Parent $executable) 'Agent.toml'
if (-not (Test-Path -LiteralPath $configPath -PathType Leaf)) {
    Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'Agent.example.toml') -Destination $configPath
    throw "Example configuration created at $configPath. Set server URL and token, then rerun this script."
}

New-Service -Name $serviceName -DisplayName 'Lightweight WOL Agent' `
    -BinaryPathName ('"{0}"' -f $executable) -StartupType Automatic `
    -Description 'Maintains the WOL server connection and handles remote shutdown requests.' | Out-Null
& sc.exe failure $serviceName reset= 86400 actions= restart/10000/restart/30000/restart/60000
if ($LASTEXITCODE -ne 0) { throw 'Failed to configure service recovery.' }
& sc.exe failureflag $serviceName 1
if ($LASTEXITCODE -ne 0) { throw 'Failed to enable recovery on service errors.' }
Start-Service -Name $serviceName
(Get-Service -Name $serviceName).WaitForStatus('Running', [TimeSpan]::FromSeconds(15))
Write-Host 'Service installed and started as LocalSystem. Startup type: Automatic.'
Write-Host "Configuration: $configPath"
Write-Host "Log: $(Join-Path (Split-Path -Parent $executable) 'Agent.log')"
