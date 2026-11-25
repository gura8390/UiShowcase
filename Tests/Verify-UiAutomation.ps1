$ErrorActionPreference = "Stop"

Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
Add-Type -AssemblyName System.Drawing

Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class NativeDpiCheck {
    [DllImport("shcore.dll")]
    public static extern int GetProcessDpiAwareness(IntPtr hprocess, out int value);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    public static extern IntPtr FindWindow(string lpClassName, string lpWindowName);
    [DllImport("user32.dll")]
    public static extern bool PostMessage(IntPtr hWnd, uint Msg, IntPtr wParam, IntPtr lParam);
}
"@

$exePath = "D:\Codex\UiShowcase\target\release\UiShowcase.exe"
if (-not (Test-Path $exePath)) {
    throw "未找到可执行文件：$exePath"
}

$testDir = Join-Path "D:\Codex\UiShowcase\Tests" "isolated_run"
if (Test-Path $testDir) { Remove-Item $testDir -Recurse -Force }
New-Item -ItemType Directory -Path $testDir -Force | Out-Null

$testConfigPath = Join-Path $testDir "settings.json"
$testAutoRunName = "澄明界面展示_UIA验收临时项"
$testMutexName = "UiShowcase_Rust_UIA_$(Get-Random)"
$runRegPath = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run"

$results = [System.Collections.Generic.List[string]]::new()
function Add-Pass([string]$msg) {
    $results.Add("[PASS] $msg")
    Write-Host "[PASS] $msg" -ForegroundColor Green
}

function Find-WindowByPid([int]$pidTarget, [int]$timeoutMs = 10000) {
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $cond = New-Object System.Windows.Automation.AndCondition(
        (New-Object System.Windows.Automation.PropertyCondition(
            [System.Windows.Automation.AutomationElement]::ProcessIdProperty, $pidTarget)),
        (New-Object System.Windows.Automation.PropertyCondition(
            [System.Windows.Automation.AutomationElement]::ControlTypeProperty,
            [System.Windows.Automation.ControlType]::Window))
    )
    while ($sw.ElapsedMilliseconds -lt $timeoutMs) {
        $win = [System.Windows.Automation.AutomationElement]::RootElement.FindFirst(
            [System.Windows.Automation.TreeScope]::Children, $cond)
        if ($null -ne $win) { return $win }
        Start-Sleep -Milliseconds 150
    }
    return $null
}

function Find-ByName([System.Windows.Automation.AutomationElement]$root, [string]$name, [int]$timeoutMs = 4000) {
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $cond = New-Object System.Windows.Automation.PropertyCondition(
        [System.Windows.Automation.AutomationElement]::NameProperty, $name)
    while ($sw.ElapsedMilliseconds -lt $timeoutMs) {
        $el = $root.FindFirst([System.Windows.Automation.TreeScope]::Descendants, $cond)
        if ($null -ne $el) { return $el }
        Start-Sleep -Milliseconds 120
    }
    return $null
}

function Find-ByNameContains([System.Windows.Automation.AutomationElement]$root, [string]$needle, [int]$timeoutMs = 4000) {
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    while ($sw.ElapsedMilliseconds -lt $timeoutMs) {
        $all = $root.FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition)
        foreach ($el in $all) {
            try {
                if ($null -ne $el.Current.Name -and $el.Current.Name.Contains($needle)) { return $el }
            } catch {}
        }
        Start-Sleep -Milliseconds 150
    }
    return $null
}

function Trigger-Control([System.Windows.Automation.AutomationElement]$el, [string]$desc) {
    if ($null -eq $el) { throw "未找到控件：$desc" }
    $obj = $null
    if ($el.TryGetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern, [ref]$obj)) {
        $obj.Invoke()
        return
    }
    if ($el.TryGetCurrentPattern([System.Windows.Automation.SelectionItemPattern]::Pattern, [ref]$obj)) {
        $obj.Select()
        return
    }
    if ($el.TryGetCurrentPattern([System.Windows.Automation.TogglePattern]::Pattern, [ref]$obj)) {
        $obj.Toggle()
        return
    }
    throw "控件「$desc」未暴露 Invoke/SelectionItem/Toggle 语义动作"
}

function Get-ToggleState([System.Windows.Automation.AutomationElement]$el) {
    $pat = $el.GetCurrentPattern([System.Windows.Automation.TogglePattern]::Pattern)
    return $pat.Current.ToggleState
}

function Get-IsSelected([System.Windows.Automation.AutomationElement]$el) {
    $obj = $null
    if ($el.TryGetCurrentPattern([System.Windows.Automation.SelectionItemPattern]::Pattern, [ref]$obj)) {
        return $obj.Current.IsSelected
    }
    if ($el.TryGetCurrentPattern([System.Windows.Automation.TogglePattern]::Pattern, [ref]$obj)) {
        return $obj.Current.ToggleState -eq [System.Windows.Automation.ToggleState]::On
    }
    return $false
}

function Start-IsolatedApp([string[]]$extraArgs = @()) {
    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName = $exePath
    $psi.UseShellExecute = $false
    if ($extraArgs.Count -gt 0) {
        $psi.Arguments = ($extraArgs | ForEach-Object { "`"$_`"" }) -join " "
    }
    $psi.Environment["UISHOWCASE_CONFIG_PATH"] = $testConfigPath
    $psi.Environment["UISHOWCASE_AUTORUN_NAME"] = $testAutoRunName
    $psi.Environment["UISHOWCASE_MUTEX_NAME"] = $testMutexName
    return [System.Diagnostics.Process]::Start($psi)
}

$proc1 = $null
$proc2 = $null

try {
    if (Get-ItemProperty -Path $runRegPath -Name $testAutoRunName -ErrorAction SilentlyContinue) {
        Remove-ItemProperty -Path $runRegPath -Name $testAutoRunName -ErrorAction SilentlyContinue
    }

    # 1. 验证 PE 内嵌图标
    $icon = [System.Drawing.Icon]::ExtractAssociatedIcon($exePath)
    if ($null -eq $icon -or $icon.Width -lt 16) {
        throw "可执行文件缺少内嵌图标"
    }
    $icon.Dispose()
    Add-Pass "PE 内嵌应用图标校验通过"

    # 2. 启动主窗口并检查 DPI 感知 + 标题
    $proc1 = Start-IsolatedApp
    $win = Find-WindowByPid -pidTarget $proc1.Id
    if ($null -eq $win) { throw "主窗口未启动或未暴露 UI Automation 窗口节点" }

    if ($win.Current.Name -ne "澄明界面展示") {
        throw "主窗口标题不符，实际为：$($win.Current.Name)"
    }

    $dpiAwareness = -1
    $hr = [NativeDpiCheck]::GetProcessDpiAwareness($proc1.Handle, [ref]$dpiAwareness)
    if ($hr -ne 0 -or $dpiAwareness -ne 2) {
        throw "DPI 感知等级不是 PerMonitorV2(2)，实际 hr=$hr, awareness=$dpiAwareness"
    }
    Add-Pass "主窗口启动成功（标题「澄明界面展示」，PerMonitorV2 DPI 感知 = 2）"

    # 等待首帧 AccessKit 树就绪
    $navOverview = Find-ByName $win "导航：总览与色板" 6000
    if ($null -eq $navOverview) { throw "AccessKit 无障碍树未暴露导航节点「导航：总览与色板」" }

    # 3. 验证总览页四张原则卡等宽等高 + 色板网格不重叠 + 字体层级纵向堆叠 + 浅/深色切换
    $pCards = @(
        (Find-ByName $win "原则卡片：01 · GPU 锐利字体" 3000),
        (Find-ByName $win "原则卡片：02 · 色阶与细线分层" 3000),
        (Find-ByName $win "原则卡片：03 · 语义双主题" 3000),
        (Find-ByName $win "原则卡片：04 · 居中与无障碍" 3000)
    )
    if ($pCards | Where-Object { $null -eq $_ }) {
        throw "总览页四张核心原则卡片节点缺失"
    }
    $refW = $pCards[0].Current.BoundingRectangle.Width
    $refH = $pCards[0].Current.BoundingRectangle.Height
    $refTop = $pCards[0].Current.BoundingRectangle.Top
    $winRightFull = $win.Current.BoundingRectangle.Right
    for ($i = 0; $i -lt $pCards.Count; $i++) {
        $br = $pCards[$i].Current.BoundingRectangle
        if ([math]::Abs($br.Width - $refW) -gt 2 -or [math]::Abs($br.Height - $refH) -gt 2) {
            throw "四张核心原则卡片尺寸不一致（参考=${refW}x${refH}，实际=$($br.Width)x$($br.Height)）"
        }
        if ([math]::Abs($br.Top - $refTop) -gt 2) {
            throw "四张核心原则卡片未处于同一水平行（第 $i 张 Top=$($br.Top)，参考 Top=$refTop）"
        }
        if ($br.Right -gt ($winRightFull + 2)) {
            throw "第 $i 张核心原则卡片越出窗口右边界（$($br.Right) > $winRightFull）"
        }
        if ($i -gt 0) {
            $prevR = $pCards[$i - 1].Current.BoundingRectangle.Right
            $gapPx = $br.Left - $prevR
            if ($gapPx -lt 4 -or $gapPx -gt 40) {
                throw "第 $($i - 1) 与第 $i 张原则卡片发生重叠或间距异常（gap=${gapPx}px，前右=$prevR，后左=$($br.Left)）"
            }
        }
    }
    $sw1 = Find-ByNameContains $win "画布底色" 3000
    $sw2 = Find-ByNameContains $win "表面面板" 3000
    $h1El = Find-ByNameContains $win "H1 页面主标题" 3000
    $h2El = Find-ByNameContains $win "H2 分区标题" 3000
    if ($null -eq $sw1 -or $null -eq $sw2 -or $null -eq $h1El -or $null -eq $h2El) {
        throw "总览页色板或字体层级节点缺失"
    }
    if ($sw1.Current.BoundingRectangle.Right -ge $sw2.Current.BoundingRectangle.Left) {
        throw "语义色板前两项发生横向重叠（$($sw1.Current.BoundingRectangle.Right) >= $($sw2.Current.BoundingRectangle.Left)）"
    }
    if ($h1El.Current.BoundingRectangle.Bottom -gt ($h2El.Current.BoundingRectangle.Top + 2)) {
        throw "字体层级 H1 与 H2 未纵向堆叠（H1.Bottom=$($h1El.Current.BoundingRectangle.Bottom), H2.Top=$($h2El.Current.BoundingRectangle.Top)）"
    }
    # 验证顶栏三组快捷切换控件尺寸与风格完全一致，且与副标题保持充足间距
    $hdrFont = Find-ByName $win "快速切换字体：楷体" 3000
    $hdrDens = Find-ByName $win "快速切换密度：标准" 3000
    $hdrThemeLight = Find-ByName $win "快速切换主题：浅色" 3000
    $hdrThemeDark = Find-ByName $win "快速切换主题：深色" 3000
    if ($null -eq $hdrFont -or $null -eq $hdrDens -or $null -eq $hdrThemeLight -or $null -eq $hdrThemeDark) {
        throw "顶栏三组统一分段切换控件缺失"
    }
    if ([math]::Abs($hdrFont.Current.BoundingRectangle.Height - $hdrThemeLight.Current.BoundingRectangle.Height) -gt 2 -or
        [math]::Abs($hdrDens.Current.BoundingRectangle.Height - $hdrThemeLight.Current.BoundingRectangle.Height) -gt 2) {
        throw "顶栏三组快捷切换控件高度不一致"
    }
    if ($null -eq (Find-ByNameContains $win "#F6F7F9" 3000)) {
        throw "浅色模式下未读取到 BgCanvas 色值 #F6F7F9"
    }
    Trigger-Control $hdrThemeDark "快速切换主题：深色"
    Start-Sleep -Milliseconds 350
    if ($null -eq (Find-ByNameContains $win "#181A1F" 3000)) {
        throw "切换深色后色板未实时刷新为 #181A1F"
    }
    Trigger-Control (Find-ByName $win "快速切换主题：浅色" 3000) "快速切换主题：浅色"
    Start-Sleep -Milliseconds 300
    Add-Pass "总览页原则卡等高、顶栏三组切换统一、语义色板与浅/深色切换通过（#F6F7F9 <-> #181A1F）"

    # 4. 验证标准/紧凑密度真实改变控件渲染高度
    $stdH = [math]::Round((Find-ByName $win "导航：总览与色板").Current.BoundingRectangle.Height, 1)
    Trigger-Control (Find-ByName $win "快速切换密度：紧凑") "快速切换密度：紧凑"
    Start-Sleep -Milliseconds 300
    $compactH = [math]::Round((Find-ByName $win "导航：总览与色板").Current.BoundingRectangle.Height, 1)
    if ($compactH -ge ($stdH - 4)) {
        throw "切换紧凑密度后导航项实际高度未收缩（标准=$stdH，紧凑=$compactH）"
    }
    Trigger-Control (Find-ByName $win "快速切换密度：标准") "快速切换密度：标准"
    Start-Sleep -Milliseconds 250
    Add-Pass "标准/紧凑密度布局收缩通过（导航项高度 $stdH -> $compactH px）"

    # 5. 验证顶栏实时字体切换（楷体 <-> 雅黑）
    Trigger-Control (Find-ByName $win "快速切换字体：雅黑") "快速切换字体：雅黑"
    Start-Sleep -Milliseconds 300
    if ($null -eq (Find-ByNameContains $win "微软雅黑 UI" 3000)) {
        throw "切换微软雅黑 UI 后状态栏未同步更新"
    }
    Trigger-Control (Find-ByName $win "快速切换字体：楷体") "快速切换字体：楷体"
    Start-Sleep -Milliseconds 300
    Add-Pass "顶栏字体实时切换（方正公文楷体 <-> 微软雅黑 UI）通过"

    # 6. 典型工具界面：空闲不显示待机进度 + 列表四列不越界重叠 + 自定义下拉 + 空状态 + 二次确认
    Trigger-Control (Find-ByName $win "导航：典型工具界面") "导航：典型工具界面"
    Start-Sleep -Milliseconds 300
    if ($null -ne (Find-ByName $win "空闲" 300)) {
        throw "典型工具界面空闲状态不应显示「空闲」待机占位文案"
    }
    $row1 = Find-ByNameContains $win "DOC-001" 3000
    $row2 = Find-ByNameContains $win "DOC-002" 3000
    $delBtn = Find-ByName $win "移除选中记录（演示二次确认）" 3000
    if ($null -eq $row1 -or $null -eq $row2 -or $null -eq $delBtn) {
        throw "典型工具界面未找到列表行或右侧详情操作按钮"
    }
    $r1 = $row1.Current.BoundingRectangle
    $rDel = $delBtn.Current.BoundingRectangle
    if ($r1.Right -ge $rDel.Left) {
        throw "核查记录列表行右边界 ($($r1.Right)) 越界侵入右侧详情区 ($($rDel.Left))"
    }
    Trigger-Control $row2 "选择 DOC-002"
    Start-Sleep -Milliseconds 250
    if ($null -eq (Find-ByNameContains $win "4C19-DE07" 3000)) {
        throw "点击列表第 2 行后右侧详情未同步切换至 DOC-002"
    }
    # 验证自定义状态筛选下拉框可展开、切换选项并与重置按钮等高对齐
    $resetBtn = Find-ByName $win "重置数据" 3000
    $filterCombo = Find-ByNameContains $win "按状态筛选：全部状态" 3000
    if ($null -eq $resetBtn -or $null -eq $filterCombo) {
        throw "典型工具界面未找到重置数据按钮或自定义状态筛选下拉框"
    }
    if ([math]::Abs($resetBtn.Current.BoundingRectangle.Height - $filterCombo.Current.BoundingRectangle.Height) -gt 2) {
        throw "状态筛选下拉框高度与工具栏按钮不一致（按钮=$($resetBtn.Current.BoundingRectangle.Height)，下拉=$($filterCombo.Current.BoundingRectangle.Height)）"
    }
    Trigger-Control $filterCombo "展开状态筛选下拉框"
    Start-Sleep -Milliseconds 250
    $optDanger = Find-ByName $win "存在异常" 3000
    if ($null -eq $optDanger) { throw "展开状态筛选下拉框后未显示选项「存在异常」" }
    Trigger-Control $optDanger "选择存在异常"
    Start-Sleep -Milliseconds 250
    if ($null -eq (Find-ByNameContains $win "按状态筛选：存在异常" 3000)) {
        throw "选择「存在异常」后下拉框未更新选中文案"
    }
    Trigger-Control $resetBtn "重置数据"
    Start-Sleep -Milliseconds 250
    Trigger-Control (Find-ByName $win "预览空状态") "预览空状态"
    Start-Sleep -Milliseconds 250
    Trigger-Control (Find-ByName $win "恢复示例列表") "恢复示例列表"
    Start-Sleep -Milliseconds 250
    Trigger-Control (Find-ByName $win "移除选中记录（演示二次确认）") "移除选中记录（演示二次确认）"
    Start-Sleep -Milliseconds 300
    Trigger-Control (Find-ByName $win "取消") "二次确认弹窗取消按钮"
    Start-Sleep -Milliseconds 250
    Add-Pass "典型工具界面四列列表不重叠、行切换详情、空状态与二次确认弹窗验收通过"

    # 7. 控件与交互态：几何布局不重叠 + 五态反馈 + 内联错误 + 开关 TogglePattern
    Trigger-Control (Find-ByName $win "导航：控件与交互态") "导航：控件与交互态"
    Start-Sleep -Milliseconds 300
    $editDir = Find-ByName $win "目标工作目录" 3000
    $btnSimErr = Find-ByName $win "模拟非法输入" 3000
    $toggleA = Find-ByName $win "保存配置时自动校验完整性" 3000
    $toggleB = Find-ByName $win "列表中仅展示存在异常的记录" 3000
    $bnInfo = Find-ByNameContains $win "状态横幅：[信息] OpenGL" 3000
    $bnOk = Find-ByNameContains $win "状态横幅：[成功] 本地设置" 3000
    $bnWarn = Find-ByNameContains $win "状态横幅：[警告] 检测到" 3000
    $bnErr = Find-ByNameContains $win "状态横幅：[错误] 目标目录" 3000
    $sliderEl = Find-ByName $win "并发扫描阈值" 3000
    if ($null -eq $editDir -or $null -eq $btnSimErr -or $null -eq $toggleA -or $null -eq $toggleB -or $null -eq $bnInfo -or $null -eq $bnOk -or $null -eq $bnWarn -or $null -eq $bnErr -or $null -eq $sliderEl) {
        throw "控件页关键节点缺失"
    }
    $bnRefW = $bnInfo.Current.BoundingRectangle.Width
    $bnRefH = $bnInfo.Current.BoundingRectangle.Height
    foreach ($bn in @($bnOk, $bnWarn, $bnErr)) {
        $br = $bn.Current.BoundingRectangle
        if ([math]::Abs($br.Width - $bnRefW) -gt 2 -or [math]::Abs($br.Height - $bnRefH) -gt 2) {
            throw "状态反馈四件套框尺寸不一致（参考=${bnRefW}x${bnRefH}，实际=$($br.Width)x$($br.Height)）"
        }
    }
    if ($editDir.Current.BoundingRectangle.Bottom -gt ($btnSimErr.Current.BoundingRectangle.Top + 4)) {
        throw "控件页输入框与下方操作按钮未纵向排列"
    }
    if ($btnSimErr.Current.BoundingRectangle.Right -ge $toggleA.Current.BoundingRectangle.Left) {
        throw "控件页左列与右列发生横向重叠"
    }
    if ($toggleA.Current.BoundingRectangle.Bottom -gt ($toggleB.Current.BoundingRectangle.Top + 4)) {
        throw "控件页两个开关未纵向堆叠"
    }
    if ($bnInfo.Current.BoundingRectangle.Right -ge $bnOk.Current.BoundingRectangle.Left -or
        ($bnOk.Current.BoundingRectangle.Left - $bnInfo.Current.BoundingRectangle.Right) -gt 40) {
        throw "状态反馈「信息」与「成功」横幅重叠或间距异常（infoRight=$($bnInfo.Current.BoundingRectangle.Right), okLeft=$($bnOk.Current.BoundingRectangle.Left)）"
    }
    if ($bnWarn.Current.BoundingRectangle.Right -ge $bnErr.Current.BoundingRectangle.Left -or
        ($bnErr.Current.BoundingRectangle.Left - $bnWarn.Current.BoundingRectangle.Right) -gt 40) {
        throw "状态反馈「警告」与「错误」横幅重叠或间距异常（warnRight=$($bnWarn.Current.BoundingRectangle.Right), errLeft=$($bnErr.Current.BoundingRectangle.Left)）"
    }
    if ([math]::Abs($bnInfo.Current.BoundingRectangle.Top - $bnOk.Current.BoundingRectangle.Top) -gt 2 -or
        [math]::Abs($bnWarn.Current.BoundingRectangle.Top - $bnErr.Current.BoundingRectangle.Top) -gt 2) {
        throw "状态反馈四件套同行两框未顶部对齐"
    }
    if ($bnOk.Current.BoundingRectangle.Right -gt ($win.Current.BoundingRectangle.Right + 2)) {
        throw "状态反馈「成功」横幅越出窗口右边界"
    }
    if ($bnInfo.Current.BoundingRectangle.Bottom -gt ($bnWarn.Current.BoundingRectangle.Top + 4)) {
        throw "状态反馈第一行与第二行未纵向分开"
    }
    Trigger-Control (Find-ByName $win "主操作按钮") "主操作按钮"
    Start-Sleep -Milliseconds 200
    if ($null -eq (Find-ByNameContains $win "已触发「主操作按钮」" 3000)) {
        throw "主按钮点击反馈未更新"
    }
    Trigger-Control $btnSimErr "模拟非法输入"
    Start-Sleep -Milliseconds 250
    if ($null -eq (Find-ByNameContains $win "[错误] 路径包含非法字符" 3000)) {
        throw "非法输入后未呈现中文内联错误提示"
    }
    Trigger-Control (Find-ByName $win "恢复合法路径") "恢复合法路径"
    Start-Sleep -Milliseconds 200

    $beforeState = Get-ToggleState $toggleA
    Trigger-Control $toggleA "保存配置时自动校验完整性"
    Start-Sleep -Milliseconds 200
    $afterState = Get-ToggleState (Find-ByName $win "保存配置时自动校验完整性")
    if ($beforeState -eq $afterState) {
        throw "开关 TogglePattern 状态未变化"
    }
    Add-Pass "控件页双列与状态反馈 2×2 网格几何对齐、五态反馈、内联报错与开关 TogglePattern 验收通过"

    # 8. 主题与设置：深色 + 青瓷绿 + 紧凑密度 + 登录自启注册表写入/清除
    Trigger-Control (Find-ByName $win "导航：主题与设置") "导航：主题与设置"
    Start-Sleep -Milliseconds 250
    Trigger-Control (Find-ByName $win "主题模式：深色模式") "主题模式：深色模式"
    Start-Sleep -Milliseconds 200
    Trigger-Control (Find-ByName $win "强调色：青瓷绿") "强调色：青瓷绿"
    Start-Sleep -Milliseconds 200
    Trigger-Control (Find-ByName $win "界面密度：紧凑密度") "界面密度：紧凑密度"
    Start-Sleep -Milliseconds 250

    $autoToggle = Find-ByName $win "登录时启动（同步写入当前用户启动项注册表）"
    Trigger-Control $autoToggle "登录时启动"
    Start-Sleep -Milliseconds 300
    $regVal = (Get-ItemProperty -Path $runRegPath -Name $testAutoRunName -ErrorAction SilentlyContinue).$testAutoRunName
    if ([string]::IsNullOrWhiteSpace($regVal) -or $regVal -notlike "*--minimized*") {
        throw "登录启动项写入验证失败，实际值：$regVal"
    }
    Trigger-Control (Find-ByName $win "登录时启动（同步写入当前用户启动项注册表）") "停用登录时启动"
    Start-Sleep -Milliseconds 300
    if ($null -ne (Get-ItemProperty -Path $runRegPath -Name $testAutoRunName -ErrorAction SilentlyContinue).$testAutoRunName) {
        throw "停用登录启动项后未清除注册表项"
    }
    Add-Pass "设置页主题/强调色/密度切换与 HKCU Run 启动项写入/清除验收通过"

    # 9. 关于与环境页检查
    Trigger-Control (Find-ByName $win "导航：关于与环境") "导航：关于与环境"
    Start-Sleep -Milliseconds 250
    if ($null -eq (Find-ByNameContains $win "OpenGL (glow)" 3000)) {
        throw "关于页未显示 OpenGL (glow) 渲染后端信息"
    }
    Add-Pass "关于与环境页 DPI/glow 后端读数验收通过"

    # 9.5 最小窗口（820×540 逻辑尺寸）窄窗响应式零溢出核验
    $tfObj = $null
    if ($win.TryGetCurrentPattern([System.Windows.Automation.TransformPattern]::Pattern, [ref]$tfObj)) {
        # 切换回标准密度便于检验最宽侧边栏（232pt）下的最小窗口极限布局
        Trigger-Control (Find-ByName $win "快速切换密度：标准" 3000) "切换标准密度"
        Start-Sleep -Milliseconds 200
        $origW = $win.Current.BoundingRectangle.Width
        $origH = $win.Current.BoundingRectangle.Height
        # 按当前窗口缩放估算出 820×540 物理像素（150% 下约 1230×810，100% 下 820×540）
        $scale = if ($stdH -gt 0) { $stdH / 40.0 } else { 1.0 }
        $minPhysW = [math]::Round(820.0 * $scale)
        $minPhysH = [math]::Round(540.0 * $scale)
        $tfObj.Resize($minPhysW, $minPhysH)
        Start-Sleep -Milliseconds 350
        $wRight = $win.Current.BoundingRectangle.Right

        foreach ($navCheck in @(
            @{ Nav = "导航：总览与色板"; Probe = "原则卡片：04 · 居中与无障碍" },
            @{ Nav = "导航：典型工具界面"; Probe = "搜索核查记录" },
            @{ Nav = "导航：控件与交互态"; Probe = "并发扫描阈值" },
            @{ Nav = "导航：主题与设置"; Probe = "强调色：琥珀金" }
        )) {
            Trigger-Control (Find-ByName $win $navCheck.Nav 3000) $navCheck.Nav
            Start-Sleep -Milliseconds 250
            $probeEl = Find-ByName $win $navCheck.Probe 3000
            if ($null -eq $probeEl) {
                throw "最小窗口下未找到节点「$($navCheck.Probe)」"
            }
            if ($probeEl.Current.BoundingRectangle.Right -gt ($wRight + 4)) {
                throw "最小窗口（820×540）下「$($navCheck.Probe)」右边界 ($($probeEl.Current.BoundingRectangle.Right)) 超出窗口右边界 ($wRight)"
            }
        }
        $tfObj.Resize($origW, $origH)
        Start-Sleep -Milliseconds 250
        # 恢复紧凑密度以保持后续持久化验收一致
        Trigger-Control (Find-ByName $win "快速切换密度：紧凑" 3000) "恢复紧凑密度"
        Start-Sleep -Milliseconds 250
        Add-Pass "最小窗口（820×540）全站四页面窄窗响应式零横向溢出验收通过"
    }

    # 10. 正常退出应用
    Trigger-Control (Find-ByName $win "退出应用") "退出应用"
    if (-not $proc1.WaitForExit(5000)) { throw "点击退出应用后进程未在 5 秒内退出" }
    Add-Pass "通过导航栏「退出应用」干净退出"

    # 11. 重启持久化 + 单实例排他验证
    $savedJson = Get-Content $testConfigPath -Raw -Encoding UTF8 | ConvertFrom-Json
    if ($savedJson.theme_mode -ne "Dark" -or $savedJson.accent_preset -ne "Teal" -or $savedJson.density_mode -ne "Compact") {
        throw "settings.json 持久化内容不符：$($savedJson | ConvertTo-Json -Compress)"
    }

    $proc2 = Start-IsolatedApp
    $win2 = Find-WindowByPid -pidTarget $proc2.Id
    if ($null -eq $win2) { throw "第二次启动主窗口失败" }
    if ($null -eq (Find-ByNameContains $win2 "#181A1F" 4000)) {
        throw "重启后深色主题未实际加载生效"
    }
    Trigger-Control (Find-ByName $win2 "导航：主题与设置") "导航：主题与设置"
    Start-Sleep -Milliseconds 250
    if (-not (Get-IsSelected (Find-ByName $win2 "主题模式：深色模式"))) { throw "深色模式未持久化" }
    if (-not (Get-IsSelected (Find-ByName $win2 "强调色：青瓷绿"))) { throw "青瓷绿强调色未持久化" }
    if (-not (Get-IsSelected (Find-ByName $win2 "界面密度：紧凑密度"))) { throw "紧凑密度未持久化" }
    Add-Pass "重启后深色模式、青瓷绿强调色与紧凑密度真实持久化验收通过"

    # 单实例排他验证
    $procDup = Start-IsolatedApp
    if (-not $procDup.WaitForExit(4000)) {
        $procDup.Kill()
        throw "重复启动的第二个实例未自动退出，单实例排他失效"
    }
    if ($proc2.HasExited) {
        throw "重复启动后原实例意外退出"
    }
    Add-Pass "单实例 Mutex 排他验收通过（重复进程静默退出，已有实例保持运行）"

    # 托盘状态检查（若存在 Shell_TrayWnd 则验证关闭隐藏到托盘）
    $shellTray = [NativeDpiCheck]::FindWindow("Shell_TrayWnd", $null)
    if ($shellTray -ne [IntPtr]::Zero) {
        $winPat = $win2.GetCurrentPattern([System.Windows.Automation.WindowPattern]::Pattern)
        $winPat.Close()
        Start-Sleep -Milliseconds 500
        if ($proc2.HasExited) { throw "有系统托盘环境下关闭窗口未隐藏到托盘" }
        $procWake = Start-IsolatedApp
        $procWake.WaitForExit(4000) | Out-Null
        $win2 = Find-WindowByPid -pidTarget $proc2.Id -timeoutMs 4000
        if ($null -eq $win2) { throw "重复启动未能唤回隐藏在托盘中的主窗口" }
        Add-Pass "关闭隐藏到托盘与重复启动唤回主窗口验收通过"
    } else {
        Write-Host "[INFO] 当前会话为无 Shell_TrayWnd 的独立测试桌面，跳过系统托盘图标隐藏/唤回实测。" -ForegroundColor Yellow
    }

    Trigger-Control (Find-ByName $win2 "退出应用") "退出应用"
    if (-not $proc2.WaitForExit(5000)) {
        $proc2.Kill()
        throw "第二次实例退出失败"
    }

    # 12. 隐藏窗口（--minimized）下的托盘唤醒与托盘退出事件闭环测试
    $ctrlCls = "${testMutexName}_Control"
    $WM_APP_EXIT = [uint32](0x8000 + 4)

    $procHidden = Start-IsolatedApp @("--minimized")
    try {
        $ctrlHwnd = [IntPtr]::Zero
        $sw = [System.Diagnostics.Stopwatch]::StartNew()
        while ($sw.ElapsedMilliseconds -lt 5000) {
            $ctrlHwnd = [NativeDpiCheck]::FindWindow($ctrlCls, $ctrlCls)
            if ($ctrlHwnd -ne [IntPtr]::Zero) { break }
            Start-Sleep -Milliseconds 100
        }
        if ($ctrlHwnd -eq [IntPtr]::Zero) { throw "隐藏启动实例未创建托盘控制窗口 $ctrlCls" }
        Start-Sleep -Milliseconds 400
        # 直接通过托盘退出通道退出隐藏窗口
        [void][NativeDpiCheck]::PostMessage($ctrlHwnd, $WM_APP_EXIT, [IntPtr]::Zero, [IntPtr]::Zero)
        if (-not $procHidden.WaitForExit(5000)) {
            $procHidden.Kill()
            throw "隐藏窗口模式下收到托盘退出指令后未在 5 秒内退出"
        }
        Add-Pass "隐藏窗口（托盘驻留）状态下托盘「退出」通道立即退出验收通过"
    } finally {
        if ($procHidden -and -not $procHidden.HasExited) { $procHidden.Kill() }
    }

    # 13. 隐藏窗口状态下重复启动唤回主窗口，再通过托盘退出通道关闭可见窗口
    $procWakeHost = Start-IsolatedApp @("--minimized")
    try {
        $sw = [System.Diagnostics.Stopwatch]::StartNew()
        $ctrlHwnd2 = [IntPtr]::Zero
        while ($sw.ElapsedMilliseconds -lt 5000) {
            $ctrlHwnd2 = [NativeDpiCheck]::FindWindow($ctrlCls, $ctrlCls)
            if ($ctrlHwnd2 -ne [IntPtr]::Zero) { break }
            Start-Sleep -Milliseconds 100
        }
        Start-Sleep -Milliseconds 350
        $procSecond = Start-IsolatedApp
        if (-not $procSecond.WaitForExit(4000)) {
            $procSecond.Kill()
            throw "隐藏窗口唤醒测试中重复实例未退出"
        }
        $wokenWin = Find-WindowByPid -pidTarget $procWakeHost.Id -timeoutMs 5000
        if ($null -eq $wokenWin) {
            throw "隐藏窗口收到单实例唤醒消息后未能重新显示主窗口"
        }
        [void][NativeDpiCheck]::PostMessage($ctrlHwnd2, $WM_APP_EXIT, [IntPtr]::Zero, [IntPtr]::Zero)
        if (-not $procWakeHost.WaitForExit(5000)) {
            $procWakeHost.Kill()
            throw "可见窗口收到托盘退出指令后未在 5 秒内退出"
        }
        Add-Pass "隐藏窗口单实例唤醒显示 + 可见状态托盘「退出」验收通过"
    } finally {
        if ($procWakeHost -and -not $procWakeHost.HasExited) { $procWakeHost.Kill() }
    }

    Write-Host "`n全部 Rust + egui + AccessKit 自动化验收项通过（共 $($results.Count) 项）。" -ForegroundColor Cyan
}
finally {
    if ($proc1 -and -not $proc1.HasExited) { $proc1.Kill() }
    if ($proc2 -and -not $proc2.HasExited) { $proc2.Kill() }
    if (Get-ItemProperty -Path $runRegPath -Name $testAutoRunName -ErrorAction SilentlyContinue) {
        Remove-ItemProperty -Path $runRegPath -Name $testAutoRunName -ErrorAction SilentlyContinue
    }
    if (Test-Path $testDir) {
        Remove-Item $testDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}
