internal sealed class DumpOptions
{
    public string Input { get; init; } = string.Empty;
}

internal sealed class DecryptOptions
{
    public string Bpid { get; init; } = string.Empty;

    public string? PositionalInputPath { get; init; }

    public string? InputPath { get; init; }

    public string? OutputPath { get; init; }
}
