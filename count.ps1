$base = (Get-Location).Path
Get-ChildItem -Recurse -Filter *.rs -File |
    Where-Object { $_.FullName -notmatch '\\target\\' } |
    Group-Object DirectoryName |
    ForEach-Object {
        $relative = ($_.Name -replace [regex]::Escape($base), '').TrimStart('\')
        "$relative: $((Get-Content ($_.Group | Select-Object -ExpandProperty FullName) | Measure-Object -Line).Lines)"
    } | Out-File lines.txt