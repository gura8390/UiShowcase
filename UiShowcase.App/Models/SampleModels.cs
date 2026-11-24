namespace UiShowcase.Models;

public sealed class DesignTokenItem
{
    public string TokenName { get; init; } = string.Empty;
    public string DisplayName { get; init; } = string.Empty;
    public string HexValue { get; init; } = string.Empty;
    public string Usage { get; init; } = string.Empty;
    public string BrushKey { get; init; } = string.Empty;
}

public sealed class SampleTaskItem
{
    public string Id { get; init; } = string.Empty;
    public string FileName { get; init; } = string.Empty;
    public string FolderPath { get; init; } = string.Empty;
    public string SizeText { get; init; } = string.Empty;
    public string ModifiedAt { get; init; } = string.Empty;
    public string Status { get; set; } = string.Empty;
    public string StatusKind { get; set; } = "Info"; // Success, Warning, Danger, Info
    public string DetailNote { get; init; } = string.Empty;
    public string ShaShort { get; init; } = string.Empty;
}
