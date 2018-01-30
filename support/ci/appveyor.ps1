
function Get-RepoRoot {
    (Resolve-Path "$PSScriptRoot\..\..\").Path
}

function Test-ComponentChanged ($path) {
    $component = $path -replace 'components/(\w+-*\w*)/.*$', '$1'
    ($env:HAB_COMPONENTS -split ';') -contains $component
}

function Test-PullRequest() {
    ($env:APPVEYOR_REPO_BRANCH -like 'master') -and
        (test-path env:/APPVEYOR_PULL_REQUEST_NUMBER) -and
        (-not [string]::IsNullOrEmpty($env:APPVEYOR_PULL_REQUEST_NUMBER))
}

function Test-SentinelBuild() {
    $env:APPVEYOR_REPO_BRANCH -like 'sentinel*'
}

function Test-SourceChanged {
    $files = if (Test-PullRequest -or Test-SentinelBuild) {
        # for pull requests or sentinel builds diff
        # against master
        git diff master --name-only
    } else {
        # for master builds, check against the last merge
        git show :/^Merge --pretty=format:%H -m --name-only
    }

    $BuildFiles = "appveyor.yml", "build.ps1", "support/ci/appveyor.ps1", "support/ci/appveyor.bat",
                  "Cargo.toml", "Cargo.lock"
    ($files |
        where-object {
            ($BuildFiles -contains $_ ) -or
            (($_ -like 'components/*') -and
                (Test-ComponentChanged $_))
        }
    ).count -ge 1
}

pushd (Get-RepoRoot)
Write-Host "Configuring build environment"
./build.ps1 -Configure -SkipBuild

write-host "TAG: $env:APPVEYOR_REPO_TAG_NAME"
if ((Test-SourceChanged) -or (test-path env:HAB_FORCE_TEST)) {
    foreach ($BuildAction in ($env:hab_build_action -split ';')) {
        foreach ($component in ($env:hab_components -split ';')) {
            pushd "$(Get-RepoRoot)/components/$component"
            Write-Host "Testing $component"
            Write-Host ""
            cargo test --verbose
            if ($LASTEXITCODE -ne 0) {exit $LASTEXITCODE}
            popd
        }
    }
}
else {
    Write-Host "Nothing changed in Windows ported crates."
}
