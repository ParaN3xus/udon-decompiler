using AssetsTools.NET;
using AssetsTools.NET.Extra;

internal static partial class Program
{
    private const string SerializedUdonProgramAssetClassName = "SerializedUdonProgramAsset";
    private const string CompressedProgramFieldPath = "serializedProgramCompressedBytes.Array";
    private static readonly HashSet<char> InvalidFileNameChars = [..Path.GetInvalidFileNameChars()];

    private static DumpResult DumpProgramsFromBundle(string inputPath)
    {
        var fullInputPath = Path.GetFullPath(inputPath);
        var outputDirectory = BuildOutputDirectory(fullInputPath);
        Directory.CreateDirectory(outputDirectory);

        var dumpCount = 0;
        var assetsFileCount = 0;
        var usedOutputNames = new HashSet<string>(StringComparer.OrdinalIgnoreCase);

        var manager = new AssetsManager
        {
            UseQuickLookup = true,
            UseTemplateFieldCache = true,
            UseMonoTemplateFieldCache = true,
            UseRefTypeManagerCache = true
        };

        try
        {
            var bundle = manager.LoadBundleFile(fullInputPath, unpackIfPacked: true)
                ?? throw new InvalidOperationException("Failed to load the asset bundle.");

            var dirInfos = bundle.file.BlockAndDirInfo.DirectoryInfos;
            for (var bundleIndex = 0; bundleIndex < dirInfos.Count; bundleIndex++)
            {
                var assetsFile = manager.LoadAssetsFileFromBundle(bundle, bundleIndex, loadDeps: true);
                if (assetsFile == null)
                {
                    continue;
                }

                assetsFileCount++;
                var assetsFileData = assetsFile.file;
                var serializedUdonProgramScriptCache = new Dictionary<ushort, bool>();

                foreach (var assetInfo in assetsFileData.GetAssetsOfType(AssetClassID.MonoBehaviour))
                {
                    if (!IsSerializedUdonProgramAsset(
                            manager,
                            assetsFile,
                            assetInfo,
                            serializedUdonProgramScriptCache))
                    {
                        continue;
                    }

                    var baseField = manager.GetBaseField(assetsFile, assetInfo);
                    var compressedField = baseField[CompressedProgramFieldPath];
                    if (compressedField.IsDummy)
                    {
                        continue;
                    }

                    var compressedBytes = ReadByteArrayField(compressedField);
                    if (compressedBytes.Length == 0)
                    {
                        continue;
                    }

                    var assetName = GetAssetName(baseField, assetInfo);
                    var outputPath = GetUniqueOutputPath(outputDirectory, assetName, ".hex", usedOutputNames);
                    File.WriteAllText(outputPath, Convert.ToHexString(compressedBytes).ToLowerInvariant());
                    dumpCount++;
                }
            }
        }
        finally
        {
            manager.UnloadAll(unloadClassData: true);
        }

        if (assetsFileCount == 0)
        {
            throw new InvalidOperationException(
                "The bundle did not contain any readable Unity assets files.");
        }

        if (dumpCount == 0)
        {
            throw new InvalidOperationException(
                "No SerializedUdonProgramAsset objects were found in the bundle.");
        }

        return new DumpResult(outputDirectory, dumpCount);
    }

    private static bool IsSerializedUdonProgramAsset(
        AssetsManager manager,
        AssetsFileInstance assetsFile,
        AssetFileInfo assetInfo,
        Dictionary<ushort, bool> serializedUdonProgramScriptCache)
    {
        if (assetInfo.TypeId != (int)AssetClassID.MonoBehaviour)
        {
            return false;
        }

        ushort scriptIndex;
        try
        {
            scriptIndex = assetInfo.GetScriptIndex(assetsFile.file);
        }
        catch
        {
            return false;
        }

        if (scriptIndex == ushort.MaxValue)
        {
            return false;
        }

        if (serializedUdonProgramScriptCache.TryGetValue(scriptIndex, out var cachedResult))
        {
            return cachedResult;
        }

        if (TryGetTypeTreeRootClassName(assetsFile, scriptIndex, out var typeTreeClassName) &&
            string.Equals(typeTreeClassName, SerializedUdonProgramAssetClassName, StringComparison.Ordinal))
        {
            serializedUdonProgramScriptCache[scriptIndex] = true;
            return true;
        }

        AssetTypeReference? scriptInfo;
        try
        {
            scriptInfo = AssetHelper.GetAssetsFileScriptInfo(manager, assetsFile, scriptIndex);
        }
        catch
        {
            return false;
        }

        var isSerializedUdonProgram = string.Equals(
            scriptInfo?.ClassName,
            SerializedUdonProgramAssetClassName,
            StringComparison.Ordinal);
        serializedUdonProgramScriptCache[scriptIndex] = isSerializedUdonProgram;
        return isSerializedUdonProgram;
    }

    private static bool TryGetTypeTreeRootClassName(
        AssetsFileInstance assetsFile,
        ushort scriptIndex,
        out string className)
    {
        className = string.Empty;

        var typeTreeType = assetsFile.file.Metadata.FindTypeTreeTypeByID(
            (int)AssetClassID.MonoBehaviour,
            scriptIndex);

        if (typeTreeType == null ||
            typeTreeType.Nodes.Count == 0 ||
            typeTreeType.StringBufferBytes == null)
        {
            return false;
        }

        var rootTypeName = typeTreeType.Nodes[0].GetTypeString(typeTreeType.StringBufferBytes);
        if (string.IsNullOrWhiteSpace(rootTypeName) ||
            string.Equals(rootTypeName, "MonoBehaviour", StringComparison.Ordinal))
        {
            return false;
        }

        className = rootTypeName;
        return true;
    }

    private static byte[] ReadByteArrayField(AssetTypeValueField dataField)
    {
        if (dataField.IsDummy)
        {
            throw new InvalidOperationException("The byte-array field is missing.");
        }

        if (dataField.TemplateField.ValueType == AssetValueType.ByteArray)
        {
            return dataField.AsByteArray;
        }

        if (dataField.TemplateField.IsArray)
        {
            var count = dataField.Children.Count;
            var bytes = new byte[count];
            for (var index = 0; index < count; index++)
            {
                bytes[index] = dataField[index].AsByte;
            }
            return bytes;
        }

        throw new InvalidOperationException(
            $"Unsupported field value type {dataField.TemplateField.ValueType}.");
    }

    private static string GetAssetName(AssetTypeValueField baseField, AssetFileInfo assetInfo)
    {
        try
        {
            var nameField = baseField["m_Name"];
            if (!nameField.IsDummy && !string.IsNullOrWhiteSpace(nameField.AsString))
            {
                return nameField.AsString;
            }
        }
        catch
        {
        }

        // fallback
        return $"pathid_{assetInfo.PathId}";
    }

    private static string BuildOutputDirectory(string inputPath)
    {
        var parentDirectory = Path.GetDirectoryName(inputPath)
            ?? Directory.GetCurrentDirectory();
        var stem = Path.GetFileNameWithoutExtension(inputPath);
        return Path.Combine(parentDirectory, $"{stem}-dumped-programs");
    }

    private static string GetUniqueOutputPath(
        string directory,
        string rawFileNameStem,
        string extension,
        ISet<string> usedNames)
    {
        var fileNameStem = SanitizeFileName(rawFileNameStem);
        if (string.IsNullOrWhiteSpace(fileNameStem))
        {
            fileNameStem = "unnamed";
        }

        var candidateName = $"{fileNameStem}{extension}";
        if (usedNames.Add(candidateName))
        {
            return Path.Combine(directory, candidateName);
        }

        for (var index = 2; index < int.MaxValue; index++)
        {
            candidateName = $"{fileNameStem}-{index}{extension}";
            if (usedNames.Add(candidateName))
            {
                return Path.Combine(directory, candidateName);
            }
        }

        throw new InvalidOperationException($"Failed to generate a unique output path for '{fileNameStem}'.");
    }

    private static string SanitizeFileName(string value)
    {
        var cleanedChars = value
            .Trim()
            .Select(ch => InvalidFileNameChars.Contains(ch) ? '_' : ch)
            .ToArray();

        return new string(cleanedChars);
    }

    private sealed record DumpResult(string OutputDirectory, int DumpedCount);
}
