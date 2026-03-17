internal static partial class Program
{
    public static int Main(string[] args)
    {
        if (!AreValidInputs(args))
        {
            PrintUsage();
            return 1;
        }

        var hadFailure = false;
        foreach (var input in args)
        {
            try
            {
                var result = DumpProgramsFromBundle(input);
                Console.WriteLine(
                    $"[{Path.GetFileName(input)}] dumped {result.DumpedCount} program(s) " +
                    $"to {result.OutputDirectory}");
            }
            catch (Exception ex)
            {
                hadFailure = true;
                Console.Error.WriteLine($"[{input}] {ex.Message}");
            }
        }

        return hadFailure ? 1 : 0;
    }

    private static bool AreValidInputs(string[] args)
    {
        if (args.Length == 0)
        {
            return false;
        }

        foreach (var input in args)
        {
            if (!File.Exists(input))
            {
                return false;
            }
        }

        return true;
    }

    private static void PrintUsage()
    {
        var fileName = Path.GetFileName(Environment.ProcessPath);
        if (string.IsNullOrWhiteSpace(fileName))
        {
            fileName = AppDomain.CurrentDomain.FriendlyName;
        }

        Console.WriteLine($"Usage: {fileName} <world1.vrcw> [world2.vrcw] ...");
    }
}
