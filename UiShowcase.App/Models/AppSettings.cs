namespace UiShowcase.Models;

public enum ThemeMode
{
    System,
    Light,
    Dark
}

public enum DensityMode
{
    Standard,
    Compact
}

public enum AccentPreset
{
    Blue,
    Teal,
    Indigo,
    Amber
}

public sealed class AppSettings
{
    public ThemeMode ThemeMode { get; set; } = ThemeMode.Light;
    public DensityMode DensityMode { get; set; } = DensityMode.Standard;
    public AccentPreset AccentPreset { get; set; } = AccentPreset.Blue;
    public bool CloseToTray { get; set; } = true;
    public bool StartMinimizedOnAutoStart { get; set; } = true;
}
