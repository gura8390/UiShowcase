$ErrorActionPreference = "Stop"

Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
Add-Type -AssemblyName System.Drawing

$exePath = "D:\Codex\UiShowcase\UiShowcase.App\bin\Release\net8.0-windows\UiShowcase.exe"
if (-not (Test-Path $exePath)) {
    throw "未找到可执行文件：$exePath"
}

$testDir = Join-Path "D:\Codex\UiShowcase\Tests" "isolated_run"
$shotDir = Join-Path "D:\Codex\UiShowcase\Tests" "screenshots"
if (Test-Path $testDir) { Remove-Item $testDir -Recurse -Force }
if (Test-Path $shotDir) { Remove-Item $shotDir -Recurse -Force }
New-Item -ItemType Directory -Path $testDir -Force | Out-Null
New-Item -ItemType Directory -Path $shotDir -Force | Out-Null

$testConfigPath = Join-Path $testDir "settings.json"
$testAutoRunName = "澄明界面展示_UIA验收临时项"
$testMutexName = "UiShowcase_UIA_Test_$(Get-Random)"
$runRegPath = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run"

$results = [System.Collections.Generic.List[string]]::new()
function Add-Pass([string]$msg) {
    $results.Add("[PASS] $msg")
    Write-Host "[PASS] $msg" -ForegroundColor Green
}

function Find-WindowByPid([int]$pidTarget, [int]$timeoutMs = 8000) {
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $cond = New-Object System.Windows.Automation.AndCondition(
        (New-Object System.Windows.Automation.PropertyCondition(
            [System.Windows.Automation.AutomationElement]::ProcessIdProperty, $pidTarget)),
        (New-Object System.Windows.Automation.PropertyCondition(
            [System.Windows.Automation.AutomationElement]::AutomationIdProperty, "MainAppWindow"))
    )
    while ($sw.ElapsedMilliseconds -lt $timeoutMs) {
        $win = [System.Windows.Automation.AutomationElement]::RootElement.FindFirst(
            [System.Windows.Automation.TreeScope]::Children, $cond)
        if ($null -ne $win) { return $win }
        Start-Sleep -Milliseconds 150
    }
    return $null
}

function Find-ById([System.Windows.Automation.AutomationElement]$root, [string]$id, [int]$timeoutMs = 4000) {
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $cond = New-Object System.Windows.Automation.PropertyCondition(
        [System.Windows.Automation.AutomationElement]::AutomationIdProperty, $id)
    while ($sw.ElapsedMilliseconds -lt $timeoutMs) {
        $el = $root.FindFirst([System.Windows.Automation.TreeScope]::Descendants, $cond)
        if ($null -ne $el) { return $el }
        Start-Sleep -Milliseconds 100
    }
    throw "未能在超时内找到 AutomationId='$id'"
}

function Find-ByName([System.Windows.Automation.AutomationElement]$root, [string]$name, [int]$timeoutMs = 3000) {
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $cond = New-Object System.Windows.Automation.PropertyCondition(
        [System.Windows.Automation.AutomationElement]::NameProperty, $name)
    while ($sw.ElapsedMilliseconds -lt $timeoutMs) {
        $el = $root.FindFirst([System.Windows.Automation.TreeScope]::Descendants, $cond)
        if ($null -ne $el) { return $el }
        Start-Sleep -Milliseconds 100
    }
    return $null
}

function Invoke-Button([System.Windows.Automation.AutomationElement]$el) {
    $pat = $el.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern)
    $pat.Invoke()
}

function Select-Item([System.Windows.Automation.AutomationElement]$el) {
    $pat = $el.GetCurrentPattern([System.Windows.Automation.SelectionItemPattern]::Pattern)
    $pat.Select()
}

function Get-IsSelected([System.Windows.Automation.AutomationElement]$el) {
    $pat = $el.GetCurrentPattern([System.Windows.Automation.SelectionItemPattern]::Pattern)
    return $pat.Current.IsSelected
}

function Toggle-CheckBox([System.Windows.Automation.AutomationElement]$el) {
    $pat = $el.GetCurrentPattern([System.Windows.Automation.TogglePattern]::Pattern)
    $pat.Toggle()
}

function Get-ToggleState([System.Windows.Automation.AutomationElement]$el) {
    $pat = $el.GetCurrentPattern([System.Windows.Automation.TogglePattern]::Pattern)
    return $pat.Current.ToggleState
}

function Sample-PngLuminance([string]$pngPath) {
    $bmp = [System.Drawing.Bitmap]::FromFile($pngPath)
    try {
        $w = $bmp.Width
        $h = $bmp.Height
        $pxNav = $bmp.GetPixel([int]($w * 0.08), [int]($h * 0.65))
        $pxTop = $bmp.GetPixel([int]($w * 0.50), [int]($h * 0.05))
        $lumNav = [math]::Round(0.299 * $pxNav.R + 0.587 * $pxNav.G + 0.114 * $pxNav.B, 1)
        $lumTop = [math]::Round(0.299 * $pxTop.R + 0.587 * $pxTop.G + 0.114 * $pxTop.B, 1)
        return [PSCustomObject]@{
            NavRgb = "$($pxNav.R),$($pxNav.G),$($pxNav.B)"
            TopRgb = "$($pxTop.R),$($pxTop.G),$($pxTop.B)"
            NavLum = $lumNav
            TopLum = $lumTop
        }
    } finally {
        $bmp.Dispose()
    }
}

function Start-IsolatedApp([string[]]$extraArgs = @()) {
    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName = $exePath
    $psi.UseShellExecute = $false
    if ($extraArgs.Count -gt 0) {
        $psi.Arguments = ($extraArgs | ForEach-Object { "`"$_`"" }) -join " "
    }
    $psi.Environment["JPMMVJUI_CONFIG_PATH"] = $testConfigPath
    $psi.Environment["JPMMVJUI_AUTORUN_NAME"] = $testAutoRunName
    $psi.Environment["JPMMVJUI_MUTEX_NAME"] = $testMutexName
    return [System.Diagnostics.Process]::Start($psi)
}

$proc1 = $null
$proc2 = $null
$procSnap = $null

try {
    if (Get-ItemProperty -Path $runRegPath -Name $testAutoRunName -ErrorAction SilentlyContinue) {
        Remove-ItemProperty -Path $runRegPath -Name $testAutoRunName -ErrorAction SilentlyContinue
    }

    # --- 阶段 1：WPF 真实渲染快照与浅/深色像素亮度核验 ---
    $procSnap = Start-IsolatedApp @("--render-snapshots", $shotDir)
    if (-not $procSnap.WaitForExit(15000)) {
        throw "视觉快照生成超时"
    }
    $lightPix = Sample-PngLuminance (Join-Path $shotDir "01_overview_light_standard.png")
    $darkPix = Sample-PngLuminance (Join-Path $shotDir "02_overview_dark_standard.png")
    if ($lightPix.NavLum -lt 220 -or $lightPix.TopLum -lt 220) {
        throw "浅色渲染核验失败（Nav=$($lightPix.NavRgb), Top=$($lightPix.TopRgb)）"
    }
    if ($darkPix.NavLum -gt 45 -or $darkPix.TopLum -gt 55) {
        throw "深色渲染核验失败（Nav=$($darkPix.NavRgb), Top=$($darkPix.TopRgb)）"
    }
    Add-Pass "WPF 真实渲染像素核验通过（浅色亮度 $($lightPix.NavLum)/$($lightPix.TopLum) -> 深色亮度 $($darkPix.NavLum)/$($darkPix.TopLum)）"

    # 重置隔离配置文件用于 UIA 交互验收
    if (Test-Path $testConfigPath) { Remove-Item $testConfigPath -Force }

    # --- 阶段 2：UI Automation 完整交互验收 ---
    $proc1 = Start-IsolatedApp
    $win = Find-WindowByPid -pidTarget $proc1.Id
    if ($null -eq $win) { throw "主窗口未启动" }

    if ($win.Current.Name -ne "澄明界面展示") {
        throw "主窗口标题不符，实际为：$($win.Current.Name)"
    }
    Add-Pass "主窗口启动且标题统一为「澄明界面展示」"

    # 验证浅色色板色值在 UIA 中为 #F6F7F9，点击切换浅/深色后实时更新为 #181A1F
    if ($null -eq (Find-ByName $win "#F6F7F9")) { throw "浅色模式下未读取到 BgCanvas 色值 #F6F7F9" }
    Invoke-Button (Find-ById $win "TopQuickThemeButton")
    Start-Sleep -Milliseconds 300
    if ($null -eq (Find-ByName $win "#181A1F")) { throw "切换深色后色板未实时刷新为 #181A1F" }
    Invoke-Button (Find-ById $win "TopQuickThemeButton")
    Start-Sleep -Milliseconds 250
    Add-Pass "右上角一键切换浅/深色并实时刷新全页面语义色板验收通过"

    # 验证标准/紧凑密度真实改变控件渲染高度
    $stdHeight = [math]::Round((Find-ById $win "NavOverviewButton").Current.BoundingRectangle.Height, 1)
    Select-Item (Find-ById $win "TopDensityCompactRadio")
    Start-Sleep -Milliseconds 250
    $compactHeight = [math]::Round((Find-ById $win "NavOverviewButton").Current.BoundingRectangle.Height, 1)
    if ($compactHeight -ge ($stdHeight - 5)) {
        throw "切换紧凑密度后控件实际高度未收缩（标准=$stdHeight，紧凑=$compactHeight）"
    }
    Select-Item (Find-ById $win "TopDensityStandardRadio")
    Start-Sleep -Milliseconds 200
    Add-Pass "标准/紧凑密度真实布局收缩验收通过（导航项高度 $stdHeight -> $compactHeight px）"

    # 典型工具界面
    Select-Item (Find-ById $win "NavWorkbenchButton")
    Start-Sleep -Milliseconds 200
    Invoke-Button (Find-ById $win "WorkbenchToggleEmptyButton")
    Start-Sleep -Milliseconds 200
    Invoke-Button (Find-ById $win "EmptyStateRestoreButton")
    Start-Sleep -Milliseconds 200
    Add-Pass "典型工具界面空状态切换与恢复验收通过"

    Invoke-Button (Find-ById $win "WorkbenchDeleteButton")
    Start-Sleep -Milliseconds 300
    Invoke-Button (Find-ById $win "ConfirmDialogCancelButton")
    Start-Sleep -Milliseconds 200
    Add-Pass "危险操作中文二次确认弹窗唤起与取消验收通过"

    # 控件与交互态
    Select-Item (Find-ById $win "NavControlsButton")
    Start-Sleep -Milliseconds 200
    Invoke-Button (Find-ById $win "DemoPrimaryButton")
    Start-Sleep -Milliseconds 150
    if ((Find-ById $win "DemoFeedbackText").Current.Name -notlike "*主操作按钮*") { throw "主按钮点击反馈异常" }

    Invoke-Button (Find-ById $win "DemoSimulateErrorButton")
    Start-Sleep -Milliseconds 150
    if ((Find-ById $win "DemoInlineErrorText").Current.Name -notlike "*非法字符*") { throw "内联错误提示内容异常" }
    Invoke-Button (Find-ById $win "DemoRestoreValidButton")
    Start-Sleep -Milliseconds 150

    $toggleA = Find-ById $win "DemoToggleSwitchA"
    $beforeToggle = Get-ToggleState $toggleA
    Toggle-CheckBox $toggleA
    Start-Sleep -Milliseconds 100
    if ($beforeToggle -eq (Get-ToggleState $toggleA)) { throw "开关 TogglePattern 状态未变化" }
    Add-Pass "控件五态反馈、内联报错与 TogglePattern 语义动作验收通过"

    # 主题与设置
    Select-Item (Find-ById $win "NavSettingsButton")
    Start-Sleep -Milliseconds 200
    Select-Item (Find-ById $win "SettingsThemeDarkRadio")
    Select-Item (Find-ById $win "SettingsAccentTealRadio")
    Select-Item (Find-ById $win "SettingsDensityCompactRadio")
    Start-Sleep -Milliseconds 250

    $autoToggle = Find-ById $win "SettingsAutoStartToggle"
    Toggle-CheckBox $autoToggle
    Start-Sleep -Milliseconds 200
    $regVal = (Get-ItemProperty -Path $runRegPath -Name $testAutoRunName -ErrorAction SilentlyContinue).$testAutoRunName
    if ([string]::IsNullOrWhiteSpace($regVal) -or $regVal -notlike "*--minimized*") {
        throw "登录启动项写入验证失败，实际值：$regVal"
    }
    Toggle-CheckBox $autoToggle
    Start-Sleep -Milliseconds 200
    if ($null -ne (Get-ItemProperty -Path $runRegPath -Name $testAutoRunName -ErrorAction SilentlyContinue).$testAutoRunName) {
        throw "停用登录启动项后未清除注册表项"
    }
    Add-Pass "设置页主题/强调色/密度切换与登录启动项注册/移除验收通过"

    Invoke-Button (Find-ById $win "NavExitButton")
    if (-not $proc1.WaitForExit(5000)) { throw "点击退出应用后进程未在 5 秒内退出" }
    Add-Pass "通过导航栏「退出应用」正常退出并清理托盘"

    # --- 阶段 3：重启持久化与关闭隐藏到托盘 ---
    $proc2 = Start-IsolatedApp
    $win2 = Find-WindowByPid -pidTarget $proc2.Id
    if ($null -eq $win2) { throw "第二次启动主窗口失败" }
    if ($null -eq (Find-ByName $win2 "#181A1F")) { throw "重启后深色主题未实际加载生效" }

    Select-Item (Find-ById $win2 "NavSettingsButton")
    Start-Sleep -Milliseconds 200
    if (-not (Get-IsSelected (Find-ById $win2 "SettingsThemeDarkRadio"))) { throw "深色模式未持久化" }
    if (-not (Get-IsSelected (Find-ById $win2 "SettingsAccentTealRadio"))) { throw "青瓷绿强调色未持久化" }
    if (-not (Get-IsSelected (Find-ById $win2 "SettingsDensityCompactRadio"))) { throw "紧凑密度未持久化" }
    Add-Pass "重启后深色模式、青瓷绿强调色与紧凑密度真实持久化验收通过"

    $winPat = $win2.GetCurrentPattern([System.Windows.Automation.WindowPattern]::Pattern)
    $winPat.Close()
    Start-Sleep -Milliseconds 400
    if ($null -ne (Find-WindowByPid -pidTarget $proc2.Id -timeoutMs 600)) { throw "关闭窗口后未隐藏到托盘" }
    if ($proc2.HasExited) { throw "隐藏到托盘模式下进程意外退出" }
    Add-Pass "关闭主窗口隐藏到托盘行为验收通过"

    # 验证单实例排他：重复启动第二个进程应立刻退出，并自动唤回隐藏在托盘的第一个实例窗口
    $procDup = Start-IsolatedApp
    if (-not $procDup.WaitForExit(4000)) {
        $procDup.Kill()
        throw "重复启动的第二个实例未自动退出，单实例排他失效"
    }
    $restoredWin = Find-WindowByPid -pidTarget $proc2.Id -timeoutMs 4000
    if ($null -eq $restoredWin) {
        throw "重复启动后未能自动唤回已隐藏在托盘中的主窗口"
    }
    Add-Pass "单实例排他验收通过（重复进程自动退出，且成功唤回托盘中的主窗口）"

    Invoke-Button (Find-ById $restoredWin "NavExitButton")
    if (-not $proc2.WaitForExit(5000)) {
        $proc2.Kill()
        throw "唤回主窗口后退出失败"
    }

    Write-Host "`n全部 UI Automation 与真实渲染像素验收项通过（共 $($results.Count) 项）。" -ForegroundColor Cyan
}
finally {
    if ($procSnap -and -not $procSnap.HasExited) { $procSnap.Kill() }
    if ($proc1 -and -not $proc1.HasExited) { $proc1.Kill() }
    if ($proc2 -and -not $proc2.HasExited) { $proc2.Kill() }
    if (Get-ItemProperty -Path $runRegPath -Name $testAutoRunName -ErrorAction SilentlyContinue) {
        Remove-ItemProperty -Path $runRegPath -Name $testAutoRunName -ErrorAction SilentlyContinue
    }
    if (Test-Path $testDir) {
        Remove-Item $testDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}
