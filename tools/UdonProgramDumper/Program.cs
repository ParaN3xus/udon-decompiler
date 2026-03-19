using CommandLine;

internal static partial class Program
{
    public static int Main(string[] args)
    {
        return Parser.Default.ParseArguments<DumpOptions, DecryptOptions>(args).MapResult(
            (DumpOptions options) => RunDump(options),
            (DecryptOptions options) => RunDecrypt(options),
            _ => 1);
    }

    private static int RunDump(DumpOptions options)
    {
        try
        {
            var result = DumpProgramsFromBundle(options.Input);
            Console.WriteLine(
                $"[{Path.GetFileName(options.Input)}] dumped {result.DumpedCount} program(s), " +
                $"{result.DumpedVarCount} public var file(s) to {result.DumpRootDirectory}");
            return 0;
        }
        catch (Exception ex)
        {
            Console.Error.WriteLine($"[{options.Input}] {ex.Message}");
            return 1;
        }
    }

    private static int RunDecrypt(DecryptOptions options)
    {
        try
        {
            var result = DecryptBundles(options);
            Console.WriteLine($"Decrypted {result.InputPath} to {result.OutputPath}.");
            return 0;
        }
        catch (Exception ex)
        {
            Console.Error.WriteLine(ex.Message);
            return 1;
        }
    }
}
