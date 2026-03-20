using System.CommandLine;

internal static partial class Program
{
    public static int Main(string[] args)
    {
        var bundleArgument = new Argument<string>("bundle") {
            Description = "A Unity asset bundle file to dump.",
        };

        var dumpCommand = new Command(
            "dump",
            "Dump SerializedUdonProgramAsset programs from one or more Unity asset bundles.") {
            bundleArgument,
        };
        dumpCommand.SetAction(
            parseResult =>
            {
                var options = new DumpOptions { Input = parseResult.GetValue(bundleArgument)! };
                return RunDump(options);
            });

        var bpidOption = new Option<string>("--bpid", "-b") {
            Description = "The VRChat blueprint ID used to derive the decryption key.",
            Required = true,
        };
        var inputOption = new Option<string?>("--input", "-i") {
            Description = "A bundle file to decrypt.",
        };
        var outputOption = new Option<string?>("--output", "-o") {
            Description = "Output file path. Defaults to a derived sibling path.",
        };
        var positionalInputArgument = new Argument<string?>("input") {
            Description = "A bundle file to decrypt.",
            Arity = ArgumentArity.ZeroOrOne,
        };

        var decryptCommand =
            new Command("decrypt", "Decrypt VRChat asset bundle files using a BPID.") {
                bpidOption,
                inputOption,
                outputOption,
                positionalInputArgument,
            };
        decryptCommand.SetAction(parseResult =>
                                 {
                                     var options = new DecryptOptions {
                                         Bpid = parseResult.GetValue(bpidOption)!,
                                         InputPath = parseResult.GetValue(inputOption),
                                         OutputPath = parseResult.GetValue(outputOption),
                                         PositionalInputPath =
                                             parseResult.GetValue(positionalInputArgument),
                                     };
                                     return RunDecrypt(options);
                                 });

        var rootCommand =
            new RootCommand("Dump and decrypt Udon-related Unity asset bundles.") {
                dumpCommand,
                decryptCommand,
            };
        return rootCommand.Parse(args).Invoke();
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
