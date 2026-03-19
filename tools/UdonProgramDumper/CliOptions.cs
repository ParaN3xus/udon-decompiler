using CommandLine;

[Verb("dump",
      HelpText =
          "Dump SerializedUdonProgramAsset programs from one or more Unity asset bundles.")]
internal sealed class DumpOptions
{
    [Value(0, Required = true, MetaName = "bundle",
           HelpText = "A Unity asset bundle file to dump.")]
    public string Input { get; init; } = string.Empty;
}

[Verb("decrypt", HelpText = "Decrypt VRChat asset bundle files using a BPID.")]
internal sealed class DecryptOptions
{
    [Option('b', "bpid", Required = true,
            HelpText = "The VRChat blueprint ID used to derive the decryption key.")]
    public string Bpid { get; init; } = string.Empty;

    [Value(0, Required = false, MetaName = "input", HelpText = "A bundle file to decrypt.")]
    public string? PositionalInputPath { get; init; }

    [Option('i', "input", Required = false, HelpText = "A bundle file to decrypt.")]
    public string? InputPath { get; init; }

    [Option('o', "output", Required = false,
            HelpText = "Output file path. Defaults to a derived sibling path.")]
    public string? OutputPath { get; init; }
}
