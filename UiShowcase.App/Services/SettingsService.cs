using System.IO;
using System.Text.Json;
using System.Text.Json.Serialization;
using UiShowcase.Models;

namespace UiShowcase.Services;

public sealed class SettingsService
{
    private static readonly JsonSerializerOptions JsonOptions = new()
    {
        WriteIndented = true,
        Converters = { new JsonStringEnumConverter() }
    };

    public string SettingsPath { get; }

    public SettingsService()
    {
        var customPath = Environment.GetEnvironmentVariable("JPMMVJUI_CONFIG_PATH");
        if (!string.IsNullOrWhiteSpace(customPath))
        {
            SettingsPath = customPath;
        }
        else
        {
            var localAppData = Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData);
            SettingsPath = Path.Combine(localAppData, "澄明界面展示", "settings.json");
        }
    }

    public (AppSettings Settings, string? ErrorMessage) Load()
    {
        try
        {
            if (!File.Exists(SettingsPath))
            {
                return (new AppSettings(), null);
            }

            var json = File.ReadAllText(SettingsPath);
            var settings = JsonSerializer.Deserialize<AppSettings>(json, JsonOptions);
            return (settings ?? new AppSettings(), null);
        }
        catch (Exception ex)
        {
            return (new AppSettings(), $"配置读取失败，已回退为默认设置（原因：{ex.Message}）。");
        }
    }

    public (bool Success, string? ErrorMessage) Save(AppSettings settings)
    {
        try
        {
            var dir = Path.GetDirectoryName(SettingsPath);
            if (!string.IsNullOrWhiteSpace(dir) && !Directory.Exists(dir))
            {
                Directory.CreateDirectory(dir);
            }

            var json = JsonSerializer.Serialize(settings, JsonOptions);
            File.WriteAllText(SettingsPath, json);
            return (true, null);
        }
        catch (Exception ex)
        {
            return (false, $"配置保存失败：{ex.Message}");
        }
    }
}
