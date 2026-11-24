using System.Collections.ObjectModel;
using System.Windows.Media;
using UiShowcase.Models;
using UiShowcase.Services;

namespace UiShowcase.ViewModels;

public sealed class MainViewModel : ObservableObject
{
    private readonly SettingsService _settingsService;
    private readonly AutoStartService _autoStartService;
    private readonly ThemeService _themeService;
    private readonly Func<string, string, string, bool> _confirmAction;

    private CancellationTokenSource? _scanCts;
    private bool _suppressAutoSave;

    private int _selectedPageIndex;
    private ThemeMode _themeMode;
    private DensityMode _densityMode;
    private AccentPreset _accentPreset;
    private bool _closeToTray;
    private bool _startMinimizedOnAutoStart;
    private bool _autoStartEnabled;
    private string _autoStartCommandPreview = string.Empty;
    private string _settingsFeedbackText = string.Empty;
    private bool _hasSettingsError;

    private string _statusMessage = "就绪 · 澄明原生工具风设计规范展示已加载";
    private string _displayMetricsText = string.Empty;

    // 典型工具界面模拟状态
    private string _searchQuery = string.Empty;
    private string _selectedStatusFilter = "全部状态";
    private SampleTaskItem? _selectedTaskItem;
    private bool _isEmptyStateSimulated;
    private bool _isScanning;
    private int _scanProgress;
    private string _scanProgressText = "空闲";

    // 控件演示状态
    private string _demoInputText = "D:\\Codex\\UiShowcase";
    private string _demoValidationMessage = string.Empty;
    private bool _demoToggleA = true;
    private bool _demoToggleB;
    private double _demoSliderValue = 65;
    private int _demoSegmentIndex;
    private string _selectedDemoOption = "标准校验（推荐）";
    private string _demoActionFeedback = "点击下方任意按钮，或按 Tab 键体验统一 2px 键盘焦点框。";

    public MainViewModel(
        SettingsService settingsService,
        AutoStartService autoStartService,
        ThemeService themeService,
        Func<string, string, string, bool> confirmAction)
    {
        _settingsService = settingsService;
        _autoStartService = autoStartService;
        _themeService = themeService;
        _confirmAction = confirmAction;

        TokenItems = new ObservableCollection<DesignTokenItem>();
        AllTaskItems = new ObservableCollection<SampleTaskItem>(CreateInitialSampleTasks());
        FilteredTaskItems = new ObservableCollection<SampleTaskItem>();
        StatusFilterOptions = new[] { "全部状态", "已通过", "需复核", "存在异常" };
        DemoComboOptions = new[] { "标准校验（推荐）", "严格哈希比对", "仅校验文件名与大小", "快速跳过大文件" };

        StartScanCommand = new RelayCommand(StartScan, () => !IsScanning);
        CancelScanCommand = new RelayCommand(CancelScan, () => IsScanning);
        ToggleEmptyStateCommand = new RelayCommand(ToggleEmptyState);
        DeleteSelectedTaskCommand = new RelayCommand(DeleteSelectedTask, () => SelectedTaskItem is not null && !IsEmptyStateSimulated);
        ResetSampleTasksCommand = new RelayCommand(ResetSampleTasks);
        ResetSettingsCommand = new RelayCommand(ResetSettings);
        TriggerDemoFeedbackCommand = new RelayCommand(param =>
        {
            var label = param?.ToString() ?? "操作";
            DemoActionFeedback = $"已触发「{label}」（{DateTime.Now:HH:mm:ss}），状态栏与无障碍区域已同步播报。";
            StatusMessage = $"控件演示：已执行「{label}」";
        });
        QuickToggleThemeCommand = new RelayCommand(QuickToggleLightDark);

        LoadInitialState();
        RefreshFilteredTasks();
    }

    public string AppTitle => "澄明界面展示";
    public string AppVersion => "0.0.1-p7";
    public string SettingsPath => _settingsService.SettingsPath;
    public string AutoStartRegistryName => _autoStartService.ValueName;

    public ObservableCollection<DesignTokenItem> TokenItems { get; }
    public ObservableCollection<SampleTaskItem> AllTaskItems { get; }
    public ObservableCollection<SampleTaskItem> FilteredTaskItems { get; }
    public string[] StatusFilterOptions { get; }
    public string[] DemoComboOptions { get; }

    public RelayCommand StartScanCommand { get; }
    public RelayCommand CancelScanCommand { get; }
    public RelayCommand ToggleEmptyStateCommand { get; }
    public RelayCommand DeleteSelectedTaskCommand { get; }
    public RelayCommand ResetSampleTasksCommand { get; }
    public RelayCommand ResetSettingsCommand { get; }
    public RelayCommand TriggerDemoFeedbackCommand { get; }
    public RelayCommand QuickToggleThemeCommand { get; }

    public int SelectedPageIndex
    {
        get => _selectedPageIndex;
        set
        {
            if (SetProperty(ref _selectedPageIndex, value))
            {
                OnPropertyChanged(nameof(CurrentPageTitle));
                OnPropertyChanged(nameof(CurrentPageSubtitle));
                OnPropertyChanged(nameof(IsOverviewPage));
                OnPropertyChanged(nameof(IsWorkbenchPage));
                OnPropertyChanged(nameof(IsControlsPage));
                OnPropertyChanged(nameof(IsSettingsPage));
                OnPropertyChanged(nameof(IsAboutPage));
                StatusMessage = $"已切换到「{CurrentPageTitle}」";
            }
        }
    }

    public bool IsOverviewPage
    {
        get => SelectedPageIndex == 0;
        set { if (value) SelectedPageIndex = 0; }
    }

    public bool IsWorkbenchPage
    {
        get => SelectedPageIndex == 1;
        set { if (value) SelectedPageIndex = 1; }
    }

    public bool IsControlsPage
    {
        get => SelectedPageIndex == 2;
        set { if (value) SelectedPageIndex = 2; }
    }

    public bool IsSettingsPage
    {
        get => SelectedPageIndex == 3;
        set { if (value) SelectedPageIndex = 3; }
    }

    public bool IsAboutPage
    {
        get => SelectedPageIndex == 4;
        set { if (value) SelectedPageIndex = 4; }
    }

    public string CurrentPageTitle => SelectedPageIndex switch
    {
        0 => "总览与色板",
        1 => "典型工具界面",
        2 => "控件与交互态",
        3 => "主题与设置",
        4 => "关于与环境",
        _ => "总览与色板"
    };

    public string CurrentPageSubtitle => SelectedPageIndex switch
    {
        0 => "展示「澄明原生工具风」的语义色板、字号层级、等宽数据排版与双主题切换效果",
        1 => "模拟一款真实 Windows 桌面核查工具的工作台骨架、列表密度、任务进度与空状态",
        2 => "完整演示五态按钮、内联校验错误、开关、分段选择、下拉框、滑块及键盘焦点外描边",
        3 => "实时切换外观主题、强调色与密度，管理托盘行为与登录时启动（真实持久化保存）",
        4 => "统一中文标识、版本号与当前显示器的 DPI / 工作区自适应信息",
        _ => string.Empty
    };

    public ThemeMode ThemeMode
    {
        get => _themeMode;
        set
        {
            if (SetProperty(ref _themeMode, value))
            {
                OnPropertyChanged(nameof(IsThemeSystem));
                OnPropertyChanged(nameof(IsThemeLight));
                OnPropertyChanged(nameof(IsThemeDark));
                ApplyThemeAndPersist("主题模式已更新");
            }
        }
    }

    public bool IsThemeSystem
    {
        get => ThemeMode == ThemeMode.System;
        set { if (value) ThemeMode = ThemeMode.System; }
    }

    public bool IsThemeLight
    {
        get => ThemeMode == ThemeMode.Light;
        set { if (value) ThemeMode = ThemeMode.Light; }
    }

    public bool IsThemeDark
    {
        get => ThemeMode == ThemeMode.Dark;
        set { if (value) ThemeMode = ThemeMode.Dark; }
    }

    public DensityMode DensityMode
    {
        get => _densityMode;
        set
        {
            if (SetProperty(ref _densityMode, value))
            {
                OnPropertyChanged(nameof(IsDensityStandard));
                OnPropertyChanged(nameof(IsDensityCompact));
                OnPropertyChanged(nameof(DensityLabel));
                ApplyThemeAndPersist("界面密度已更新");
            }
        }
    }

    public bool IsDensityStandard
    {
        get => DensityMode == DensityMode.Standard;
        set { if (value) DensityMode = DensityMode.Standard; }
    }

    public bool IsDensityCompact
    {
        get => DensityMode == DensityMode.Compact;
        set { if (value) DensityMode = DensityMode.Compact; }
    }

    public AccentPreset AccentPreset
    {
        get => _accentPreset;
        set
        {
            if (SetProperty(ref _accentPreset, value))
            {
                OnPropertyChanged(nameof(IsAccentBlue));
                OnPropertyChanged(nameof(IsAccentTeal));
                OnPropertyChanged(nameof(IsAccentIndigo));
                OnPropertyChanged(nameof(IsAccentAmber));
                ApplyThemeAndPersist("强调色方案已更新");
            }
        }
    }

    public bool IsAccentBlue
    {
        get => AccentPreset == AccentPreset.Blue;
        set { if (value) AccentPreset = AccentPreset.Blue; }
    }

    public bool IsAccentTeal
    {
        get => AccentPreset == AccentPreset.Teal;
        set { if (value) AccentPreset = AccentPreset.Teal; }
    }

    public bool IsAccentIndigo
    {
        get => AccentPreset == AccentPreset.Indigo;
        set { if (value) AccentPreset = AccentPreset.Indigo; }
    }

    public bool IsAccentAmber
    {
        get => AccentPreset == AccentPreset.Amber;
        set { if (value) AccentPreset = AccentPreset.Amber; }
    }

    public bool CloseToTray
    {
        get => _closeToTray;
        set
        {
            if (SetProperty(ref _closeToTray, value))
            {
                SaveSettings("关闭窗口行为已保存");
            }
        }
    }

    public bool StartMinimizedOnAutoStart
    {
        get => _startMinimizedOnAutoStart;
        set
        {
            if (SetProperty(ref _startMinimizedOnAutoStart, value))
            {
                SaveSettings("登录启动显示行为已保存");
                if (AutoStartEnabled)
                {
                    var (ok, err) = _autoStartService.SetEnabled(true, value);
                    AutoStartCommandPreview = _autoStartService.GetRegisteredCommand();
                    if (!ok && err is not null)
                    {
                        SetSettingsFeedback(err, isError: true);
                    }
                }
            }
        }
    }

    public bool AutoStartEnabled
    {
        get => _autoStartEnabled;
        set
        {
            if (_autoStartEnabled == value) return;
            var (ok, err) = _autoStartService.SetEnabled(value, StartMinimizedOnAutoStart);
            if (!ok)
            {
                SetSettingsFeedback(err ?? "更新登录启动项失败。", isError: true);
                OnPropertyChanged();
                return;
            }

            _autoStartEnabled = _autoStartService.IsEnabled();
            AutoStartCommandPreview = _autoStartService.GetRegisteredCommand();
            OnPropertyChanged();
            var desc = _autoStartEnabled ? "已启用登录时启动" : "已停用登录时启动";
            SetSettingsFeedback(desc, isError: false);
            StatusMessage = desc;
        }
    }

    public string AutoStartCommandPreview
    {
        get => _autoStartCommandPreview;
        private set => SetProperty(ref _autoStartCommandPreview, value);
    }

    public string SettingsFeedbackText
    {
        get => _settingsFeedbackText;
        private set => SetProperty(ref _settingsFeedbackText, value);
    }

    public bool HasSettingsError
    {
        get => _hasSettingsError;
        private set => SetProperty(ref _hasSettingsError, value);
    }

    public string StatusMessage
    {
        get => _statusMessage;
        set => SetProperty(ref _statusMessage, value);
    }

    public string EffectiveThemeLabel => _themeService.IsHighContrast
        ? "系统高对比度"
        : (_themeService.IsEffectiveDark ? "深色模式" : "浅色模式");

    public string DensityLabel => DensityMode == DensityMode.Compact ? "紧凑密度" : "标准密度";

    public string DisplayMetricsText
    {
        get => _displayMetricsText;
        set => SetProperty(ref _displayMetricsText, value);
    }

    // 典型工具工作台属性
    public string SearchQuery
    {
        get => _searchQuery;
        set
        {
            if (SetProperty(ref _searchQuery, value))
            {
                RefreshFilteredTasks();
            }
        }
    }

    public string SelectedStatusFilter
    {
        get => _selectedStatusFilter;
        set
        {
            if (SetProperty(ref _selectedStatusFilter, value))
            {
                RefreshFilteredTasks();
            }
        }
    }

    public SampleTaskItem? SelectedTaskItem
    {
        get => _selectedTaskItem;
        set
        {
            if (SetProperty(ref _selectedTaskItem, value))
            {
                DeleteSelectedTaskCommand.RaiseCanExecuteChanged();
                OnPropertyChanged(nameof(HasSelectedTask));
            }
        }
    }

    public bool HasSelectedTask => SelectedTaskItem is not null && !IsEmptyStateSimulated;

    public bool IsEmptyStateSimulated
    {
        get => _isEmptyStateSimulated;
        set
        {
            if (SetProperty(ref _isEmptyStateSimulated, value))
            {
                OnPropertyChanged(nameof(ShowEmptyState));
                OnPropertyChanged(nameof(ShowTaskList));
                OnPropertyChanged(nameof(HasSelectedTask));
                OnPropertyChanged(nameof(EmptyStateButtonText));
                DeleteSelectedTaskCommand.RaiseCanExecuteChanged();
            }
        }
    }

    public bool ShowEmptyState => IsEmptyStateSimulated || FilteredTaskItems.Count == 0;
    public bool ShowTaskList => !ShowEmptyState;
    public string EmptyStateButtonText => IsEmptyStateSimulated ? "恢复示例列表" : "预览空状态";

    public bool IsScanning
    {
        get => _isScanning;
        set
        {
            if (SetProperty(ref _isScanning, value))
            {
                StartScanCommand.RaiseCanExecuteChanged();
                CancelScanCommand.RaiseCanExecuteChanged();
            }
        }
    }

    public int ScanProgress
    {
        get => _scanProgress;
        set => SetProperty(ref _scanProgress, value);
    }

    public string ScanProgressText
    {
        get => _scanProgressText;
        set => SetProperty(ref _scanProgressText, value);
    }

    public string TaskSummaryText =>
        $"共 {AllTaskItems.Count} 项 · 当前筛选 {FilteredTaskItems.Count} 项 · 异常 {AllTaskItems.Count(t => t.StatusKind == "Danger")} 项";

    // 控件演示属性
    public string DemoInputText
    {
        get => _demoInputText;
        set
        {
            if (SetProperty(ref _demoInputText, value))
            {
                ValidateDemoInput();
            }
        }
    }

    public string DemoValidationMessage
    {
        get => _demoValidationMessage;
        private set
        {
            if (SetProperty(ref _demoValidationMessage, value))
            {
                OnPropertyChanged(nameof(HasDemoValidationError));
            }
        }
    }

    public bool HasDemoValidationError => !string.IsNullOrEmpty(DemoValidationMessage);

    public bool DemoToggleA
    {
        get => _demoToggleA;
        set
        {
            if (SetProperty(ref _demoToggleA, value))
            {
                StatusMessage = $"示例开关「保存时自动校验」已{(value ? "开启" : "关闭")}";
            }
        }
    }

    public bool DemoToggleB
    {
        get => _demoToggleB;
        set
        {
            if (SetProperty(ref _demoToggleB, value))
            {
                StatusMessage = $"示例开关「仅显示异常记录」已{(value ? "开启" : "关闭")}";
            }
        }
    }

    public double DemoSliderValue
    {
        get => _demoSliderValue;
        set
        {
            var rounded = Math.Round(value);
            if (SetProperty(ref _demoSliderValue, rounded))
            {
                OnPropertyChanged(nameof(DemoSliderDisplay));
            }
        }
    }

    public string DemoSliderDisplay => $"{DemoSliderValue:0}%";

    public int DemoSegmentIndex
    {
        get => _demoSegmentIndex;
        set
        {
            if (SetProperty(ref _demoSegmentIndex, value))
            {
                OnPropertyChanged(nameof(IsDemoSeg0));
                OnPropertyChanged(nameof(IsDemoSeg1));
                OnPropertyChanged(nameof(IsDemoSeg2));
            }
        }
    }

    public bool IsDemoSeg0 { get => DemoSegmentIndex == 0; set { if (value) DemoSegmentIndex = 0; } }
    public bool IsDemoSeg1 { get => DemoSegmentIndex == 1; set { if (value) DemoSegmentIndex = 1; } }
    public bool IsDemoSeg2 { get => DemoSegmentIndex == 2; set { if (value) DemoSegmentIndex = 2; } }

    public string SelectedDemoOption
    {
        get => _selectedDemoOption;
        set => SetProperty(ref _selectedDemoOption, value);
    }

    public string DemoActionFeedback
    {
        get => _demoActionFeedback;
        set => SetProperty(ref _demoActionFeedback, value);
    }

    public void RefreshSystemThemeIfNeeded()
    {
        if (ThemeMode == ThemeMode.System || _themeService.IsHighContrast)
        {
            _themeService.Apply(ThemeMode, DensityMode, AccentPreset);
            OnPropertyChanged(nameof(EffectiveThemeLabel));
            RebuildDesignTokens();
        }
    }

    public void QuickToggleLightDark()
    {
        ThemeMode = _themeService.IsEffectiveDark ? ThemeMode.Light : ThemeMode.Dark;
    }

    private void LoadInitialState()
    {
        _suppressAutoSave = true;
        try
        {
            var (settings, err) = _settingsService.Load();
            _themeMode = settings.ThemeMode;
            _densityMode = settings.DensityMode;
            _accentPreset = settings.AccentPreset;
            _closeToTray = settings.CloseToTray;
            _startMinimizedOnAutoStart = settings.StartMinimizedOnAutoStart;
            _autoStartEnabled = _autoStartService.IsEnabled();
            _autoStartCommandPreview = _autoStartService.GetRegisteredCommand();

            _themeService.Apply(_themeMode, _densityMode, _accentPreset);
            RebuildDesignTokens();
            ValidateDemoInput();

            if (!string.IsNullOrEmpty(err))
            {
                SetSettingsFeedback(err, isError: true);
            }
            else
            {
                SetSettingsFeedback("已从本地配置加载真实状态。", isError: false);
            }
        }
        finally
        {
            _suppressAutoSave = false;
        }
    }

    private void ApplyThemeAndPersist(string reason)
    {
        _themeService.Apply(ThemeMode, DensityMode, AccentPreset);
        OnPropertyChanged(nameof(EffectiveThemeLabel));
        RebuildDesignTokens();
        SaveSettings(reason);
    }

    private void SaveSettings(string reason)
    {
        if (_suppressAutoSave) return;

        var settings = new AppSettings
        {
            ThemeMode = ThemeMode,
            DensityMode = DensityMode,
            AccentPreset = AccentPreset,
            CloseToTray = CloseToTray,
            StartMinimizedOnAutoStart = StartMinimizedOnAutoStart
        };

        var (ok, err) = _settingsService.Save(settings);
        if (ok)
        {
            SetSettingsFeedback($"{reason}（已保存至本地配置）", isError: false);
            StatusMessage = $"{reason} · 当前：{EffectiveThemeLabel} / {DensityLabel}";
        }
        else
        {
            SetSettingsFeedback(err ?? "配置保存失败。", isError: true);
            StatusMessage = err ?? "配置保存失败";
        }
    }

    private void SetSettingsFeedback(string message, bool isError)
    {
        SettingsFeedbackText = message;
        HasSettingsError = isError;
    }

    private void ResetSettings()
    {
        _suppressAutoSave = true;
        try
        {
            ThemeMode = ThemeMode.Light;
            DensityMode = DensityMode.Standard;
            AccentPreset = AccentPreset.Blue;
            CloseToTray = true;
            StartMinimizedOnAutoStart = true;
        }
        finally
        {
            _suppressAutoSave = false;
        }

        ApplyThemeAndPersist("已恢复默认外观与窗口设置");
    }

    private void ValidateDemoInput()
    {
        if (string.IsNullOrWhiteSpace(DemoInputText))
        {
            DemoValidationMessage = "工作目录不能为空，请输入有效的本地文件夹路径。";
            return;
        }

        if (DemoInputText.IndexOfAny(['<', '>', '"', '|', '?', '*']) >= 0)
        {
            DemoValidationMessage = "路径包含非法字符（< > \" | ? *），请修正后再继续。";
            return;
        }

        if (!DemoInputText.Contains(':') && !DemoInputText.StartsWith(@"\\"))
        {
            DemoValidationMessage = "请填写完整绝对路径，例如 D:\\Codex\\UiShowcase。";
            return;
        }

        DemoValidationMessage = string.Empty;
    }

    private void RefreshFilteredTasks()
    {
        var previousId = SelectedTaskItem?.Id;
        FilteredTaskItems.Clear();

        var query = SearchQuery.Trim();
        foreach (var item in AllTaskItems)
        {
            if (SelectedStatusFilter != "全部状态" && item.Status != SelectedStatusFilter)
            {
                continue;
            }

            if (!string.IsNullOrEmpty(query)
                && !item.FileName.Contains(query, StringComparison.OrdinalIgnoreCase)
                && !item.FolderPath.Contains(query, StringComparison.OrdinalIgnoreCase)
                && !item.Id.Contains(query, StringComparison.OrdinalIgnoreCase))
            {
                continue;
            }

            FilteredTaskItems.Add(item);
        }

        SelectedTaskItem = FilteredTaskItems.FirstOrDefault(x => x.Id == previousId)
                           ?? FilteredTaskItems.FirstOrDefault();

        OnPropertyChanged(nameof(ShowEmptyState));
        OnPropertyChanged(nameof(ShowTaskList));
        OnPropertyChanged(nameof(HasSelectedTask));
        OnPropertyChanged(nameof(TaskSummaryText));
    }

    private void ToggleEmptyState()
    {
        IsEmptyStateSimulated = !IsEmptyStateSimulated;
        StatusMessage = IsEmptyStateSimulated ? "已切换为空状态演示" : "已恢复示例任务列表";
    }

    private async void StartScan()
    {
        if (IsScanning) return;

        IsEmptyStateSimulated = false;
        IsScanning = true;
        ScanProgress = 0;
        ScanProgressText = "正在核查文件哈希（0%）...";
        StatusMessage = "模拟后台扫描已开始，界面保持可响应，可随时取消";

        _scanCts?.Dispose();
        _scanCts = new CancellationTokenSource();
        var token = _scanCts.Token;

        try
        {
            for (var p = 10; p <= 100; p += 10)
            {
                await Task.Delay(180, token);
                ScanProgress = p;
                ScanProgressText = $"正在核查文件哈希（{p}%）...";
            }

            ScanProgressText = $"已完成核查 · 共 {AllTaskItems.Count} 项（耗时 1.8 秒）";
            StatusMessage = "模拟扫描完成：界面未阻塞，进度反馈已同步更新";
        }
        catch (OperationCanceledException)
        {
            ScanProgressText = $"已在 {ScanProgress}% 处取消";
            StatusMessage = "已取消模拟扫描任务";
        }
        finally
        {
            IsScanning = false;
        }
    }

    private void CancelScan()
    {
        _scanCts?.Cancel();
    }

    private void DeleteSelectedTask()
    {
        if (SelectedTaskItem is null) return;
        var target = SelectedTaskItem;

        var confirmed = _confirmAction(
            "确认移除演示记录",
            $"确定要从当前演示列表中移除「{target.FileName}」吗？\n\n此操作仅影响界面演示数据，不会删除磁盘上的任何真实文件。",
            "确认移除");

        if (!confirmed)
        {
            StatusMessage = "已取消移除操作";
            return;
        }

        AllTaskItems.Remove(target);
        RefreshFilteredTasks();
        StatusMessage = $"已移除演示记录「{target.FileName}」";
    }

    private void ResetSampleTasks()
    {
        IsEmptyStateSimulated = false;
        SearchQuery = string.Empty;
        SelectedStatusFilter = "全部状态";
        AllTaskItems.Clear();
        foreach (var item in CreateInitialSampleTasks())
        {
            AllTaskItems.Add(item);
        }
        RefreshFilteredTasks();
        StatusMessage = "已重置典型工具界面的演示数据";
    }

    private void RebuildDesignTokens()
    {
        TokenItems.Clear();

        var specs = new (string Token, string Name, string Usage)[]
        {
            ("BgCanvasBrush", "画布底色", "窗口外层背景与侧边导航栏"),
            ("BgSurfaceBrush", "表面面板", "主内容卡片、表格与分组容器"),
            ("BgControlBrush", "控件底色", "输入框、下拉框与次按钮默认背景"),
            ("BgSelectedBrush", "选中底色", "导航选中项与列表高亮行"),
            ("TextPrimaryBrush", "主文本", "标题、正文与关键数据（对比度 ≥ 4.5:1）"),
            ("TextSecondaryBrush", "次要文本", "说明文字、辅助标签与表头"),
            ("BorderSubtleBrush", "细分隔线", "1px 区域边界与内容分割线"),
            ("AccentPrimaryBrush", "主强调色", "主操作按钮、选中指示条与焦点环"),
            ("StatusSuccessBrush", "成功语义", "校验通过、保存成功等正向反馈"),
            ("StatusWarningBrush", "警告语义", "需人工复核、兼容提示等注意状态"),
            ("StatusDangerBrush", "错误与危险", "内联报错、删除等不可逆操作警示")
        };

        var app = System.Windows.Application.Current;
        foreach (var (brushKey, name, usage) in specs)
        {
            var hex = "#------";
            if (app?.TryFindResource(brushKey) is SolidColorBrush brush)
            {
                var c = brush.Color;
                hex = $"#{c.R:X2}{c.G:X2}{c.B:X2}";
            }

            TokenItems.Add(new DesignTokenItem
            {
                TokenName = brushKey.Replace("Brush", string.Empty),
                DisplayName = name,
                HexValue = hex,
                Usage = usage,
                BrushKey = brushKey
            });
        }
    }

    private static List<SampleTaskItem> CreateInitialSampleTasks() =>
    [
        new SampleTaskItem
        {
            Id = "DOC-001",
            FileName = "2026年度项目验收汇总表.xlsx",
            FolderPath = @"D:\Codex\工作档案\验收材料",
            SizeText = "  1.42 MB",
            ModifiedAt = "2026-11-24 09:18",
            Status = "已通过",
            StatusKind = "Success",
            ShaShort = "A8F3-91C2",
            DetailNote = "结构与字段校验全部通过，可安全归档。"
        },
        new SampleTaskItem
        {
            Id = "DOC-002",
            FileName = "无障碍测试记录_主流程.json",
            FolderPath = @"D:\Codex\UiShowcase\Tests",
            SizeText = " 48.60 KB",
            ModifiedAt = "2026-11-24 11:04",
            Status = "已通过",
            StatusKind = "Success",
            ShaShort = "4C19-DE07",
            DetailNote = "UI Automation 节点名称、角色与键盘路径全部完整。"
        },
        new SampleTaskItem
        {
            Id = "DOC-003",
            FileName = "桌面应用配置备份_旧版.ini",
            FolderPath = @"D:\Codex\归档配置\2025",
            SizeText = " 12.15 KB",
            ModifiedAt = "2026-11-22 17:40",
            Status = "需复核",
            StatusKind = "Warning",
            ShaShort = "7B2E-04FA",
            DetailNote = "检测到 2 个旧版配置键，建议核对后再执行迁移。"
        },
        new SampleTaskItem
        {
            Id = "DOC-004",
            FileName = "扫描件识别缓存_重复副本.pdf",
            FolderPath = @"D:\Codex\临时导入\待清理",
            SizeText = " 18.75 MB",
            ModifiedAt = "2026-11-20 14:22",
            Status = "存在异常",
            StatusKind = "Danger",
            ShaShort = "F05D-88B1",
            DetailNote = "与归档目录中文件哈希完全一致，且缺少可提取文本层。"
        },
        new SampleTaskItem
        {
            Id = "DOC-005",
            FileName = "系统托盘与启动项规范说明.md",
            FolderPath = @"D:\Codex\规范文档",
            SizeText = "  8.94 KB",
            ModifiedAt = "2026-11-24 13:50",
            Status = "已通过",
            StatusKind = "Success",
            ShaShort = "19EA-6B3C",
            DetailNote = "UTF-8 编码正常，标题层级与链接完整。"
        }
    ];
}
