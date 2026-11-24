using System.Diagnostics;
using Microsoft.Win32;

namespace UiShowcase.Services;

public sealed class AutoStartService
{
    private const string RunKeyPath = @"Software\Microsoft\Windows\CurrentVersion\Run";
    private const string DefaultValueName = "澄明界面展示";

    public string ValueName { get; }

    public AutoStartService()
    {
        var customName = Environment.GetEnvironmentVariable("JPMMVJUI_AUTORUN_NAME");
        ValueName = string.IsNullOrWhiteSpace(customName) ? DefaultValueName : customName;
    }

    public bool IsEnabled()
    {
        try
        {
            using var key = Registry.CurrentUser.OpenSubKey(RunKeyPath, writable: false);
            var raw = key?.GetValue(ValueName) as string;
            return !string.IsNullOrWhiteSpace(raw);
        }
        catch
        {
            return false;
        }
    }

    public string GetRegisteredCommand()
    {
        try
        {
            using var key = Registry.CurrentUser.OpenSubKey(RunKeyPath, writable: false);
            return key?.GetValue(ValueName) as string ?? "未注册";
        }
        catch (Exception ex)
        {
            return $"读取失败：{ex.Message}";
        }
    }

    public (bool Success, string? ErrorMessage) SetEnabled(bool enable, bool startMinimized)
    {
        try
        {
            using var key = Registry.CurrentUser.OpenSubKey(RunKeyPath, writable: true)
                            ?? Registry.CurrentUser.CreateSubKey(RunKeyPath, writable: true);
            if (key is null)
            {
                return (false, "无法打开当前用户的启动项注册表键。");
            }

            if (enable)
            {
                var exePath = Environment.ProcessPath ?? Process.GetCurrentProcess().MainModule?.FileName;
                if (string.IsNullOrWhiteSpace(exePath))
                {
                    return (false, "无法获取当前可执行文件路径，未写入登录启动项。");
                }

                var command = startMinimized ? $"\"{exePath}\" --minimized" : $"\"{exePath}\"";
                key.SetValue(ValueName, command, RegistryValueKind.String);
            }
            else
            {
                if (key.GetValue(ValueName) is not null)
                {
                    key.DeleteValue(ValueName, throwOnMissingValue: false);
                }
            }

            return (true, null);
        }
        catch (Exception ex)
        {
            return (false, $"更新登录时启动设置失败：{ex.Message}");
        }
    }
}
