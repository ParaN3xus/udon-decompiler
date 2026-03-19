using System.Text.Json;
using AssetsTools.NET;
using AssetsTools.NET.Extra;

internal static partial class Program
{
    private const string SerializedUdonProgramAssetClassName = "SerializedUdonProgramAsset";
    private const string UdonBehaviourClassName = "UdonBehaviour";
    private const string CompressedProgramFieldPath = "serializedProgramCompressedBytes.Array";
    private const string SerializedPublicVariablesFieldPath =
        "serializedPublicVariablesBytesString";
    private const string SerializedProgramAssetPointerFieldPath = "serializedProgramAsset";
    private const string GameObjectPointerFieldPath = "m_GameObject";
    private const string PointerPathIdFieldName = "m_PathID";
    private static readonly HashSet<char> InvalidFileNameChars =
        [..Path.GetInvalidFileNameChars()];

    private static DumpResult DumpProgramsFromBundle(string inputPath)
    {
        var fullInputPath = Path.GetFullPath(inputPath);
        var dumpRootDirectory = BuildOutputDirectory(fullInputPath, "-dumped");
        var programsDirectory = Path.Combine(dumpRootDirectory, "programs");
        var varsDirectory = Path.Combine(dumpRootDirectory, "vars");
        var mapOutputPath = Path.Combine(dumpRootDirectory, "program-var-map.json");

        Directory.CreateDirectory(programsDirectory);
        Directory.CreateDirectory(varsDirectory);

        var assetsFileCount = 0;
        var programsByPathId = new Dictionary<long, ProgramDumpInfo>();
        var usedProgramNames = new HashSet<string>(StringComparer.OrdinalIgnoreCase);
        var pendingBehaviours = new List<UdonBehaviourInfo>();

        var manager = new AssetsManager {
            UseQuickLookup = true,
            UseTemplateFieldCache = true,
            UseMonoTemplateFieldCache = true,
            UseRefTypeManagerCache = true,
        };

        try
        {
            var bundle =
                manager.LoadBundleFile(fullInputPath, unpackIfPacked: true) ??
                throw new InvalidOperationException("Failed to load the asset bundle.");

            var dirInfos = bundle.file.BlockAndDirInfo.DirectoryInfos;
            for (var bundleIndex = 0; bundleIndex < dirInfos.Count; bundleIndex++)
            {
                var assetsFile =
                    manager.LoadAssetsFileFromBundle(bundle, bundleIndex, loadDeps: true);
                if (assetsFile == null)
                {
                    continue;
                }

                assetsFileCount++;
                CollectBundleData(manager, assetsFile, programsDirectory, usedProgramNames,
                                  programsByPathId, pendingBehaviours);
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

        if (programsByPathId.Count == 0)
        {
            throw new InvalidOperationException(
                "No SerializedUdonProgramAsset objects were found in the bundle.");
        }

        var programVarMap =
            BuildProgramVarMap(pendingBehaviours, programsByPathId, varsDirectory);
        WriteProgramVarMap(mapOutputPath, programVarMap);

        return new DumpResult(dumpRootDirectory, programsDirectory, varsDirectory,
                              programsByPathId.Count, pendingBehaviours.Count);
    }

    private static void CollectBundleData(AssetsManager manager, AssetsFileInstance assetsFile,
                                          string programsDirectory,
                                          ISet<string> usedProgramNames,
                                          IDictionary<long, ProgramDumpInfo> programsByPathId,
                                          ICollection<UdonBehaviourInfo> pendingBehaviours)
    {
        var assetsFileData = assetsFile.file;
        var monoScriptClassNameCache = new Dictionary<ushort, string?>();

        foreach (var assetInfo in assetsFileData.GetAssetsOfType(AssetClassID.MonoBehaviour))
        {
            var className = GetMonoBehaviourClassName(manager, assetsFile, assetInfo,
                                                      monoScriptClassNameCache);
            if (className == null)
            {
                continue;
            }

            var baseField = manager.GetBaseField(assetsFile, assetInfo);
            if (string.Equals(className, SerializedUdonProgramAssetClassName,
                              StringComparison.Ordinal))
            {
                var programInfo = TryDumpProgramAsset(baseField, assetInfo, programsDirectory,
                                                      usedProgramNames);
                if (programInfo != null)
                {
                    programsByPathId[assetInfo.PathId] = programInfo;
                }

                continue;
            }

            if (!string.Equals(className, UdonBehaviourClassName, StringComparison.Ordinal))
            {
                continue;
            }

            pendingBehaviours.Add(new UdonBehaviourInfo(
                GetPointerPathId(baseField[GameObjectPointerFieldPath]),
                GetPointerPathId(baseField[SerializedProgramAssetPointerFieldPath]),
                ReadStringField(baseField[SerializedPublicVariablesFieldPath])));
        }
    }

    private static ProgramDumpInfo? TryDumpProgramAsset(AssetTypeValueField baseField,
                                                        AssetFileInfo assetInfo,
                                                        string programsDirectory,
                                                        ISet<string> usedProgramNames)
    {
        var compressedField = baseField[CompressedProgramFieldPath];
        if (compressedField.IsDummy)
        {
            return null;
        }

        var compressedBytes = ReadByteArrayField(compressedField);
        if (compressedBytes.Length == 0)
        {
            return null;
        }

        var programHex = Convert.ToHexString(compressedBytes).ToLowerInvariant();

        var assetName = GetAssetName(baseField, assetInfo);
        var outputPath =
            GetUniqueOutputPath(programsDirectory, assetName, ".hex", usedProgramNames);
        File.WriteAllText(outputPath, programHex);
        return new ProgramDumpInfo(assetInfo.PathId,
                                   Path.GetFileNameWithoutExtension(outputPath));
    }

    private static Dictionary<string, List<ProgramVarMapEntry>> BuildProgramVarMap(
        IEnumerable<UdonBehaviourInfo> behaviours,
        IReadOnlyDictionary<long, ProgramDumpInfo> programsByPathId, string varsDirectory)
    {
        var result = new Dictionary<string, List<ProgramVarMapEntry>>(StringComparer.Ordinal);

        foreach (var behaviour in behaviours)
        {
            if (!programsByPathId.TryGetValue(behaviour.ProgramPathId, out var programInfo))
            {
                continue;
            }

            var publicVarFileName =
                $"{SanitizeFileName(programInfo.OutputName)}-{behaviour.GameObjectId}.b64";
            var publicVarOutputPath = Path.Combine(varsDirectory, publicVarFileName);
            File.WriteAllText(publicVarOutputPath, behaviour.SerializedPublicVariables);

            if (!result.TryGetValue(programInfo.OutputName, out var entries))
            {
                entries = [];
                result[programInfo.OutputName] = entries;
            }

            entries.Add(new ProgramVarMapEntry(behaviour.GameObjectId, publicVarFileName));
        }

        return result;
    }

    private static void WriteProgramVarMap(string outputPath,
                                           Dictionary<string, List<ProgramVarMapEntry>> map)
    {
        var options = new JsonSerializerOptions {
            PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
            WriteIndented = true,
        };
        File.WriteAllText(outputPath, JsonSerializer.Serialize(map, options));
    }

    private static string? GetMonoBehaviourClassName(
        AssetsManager manager, AssetsFileInstance assetsFile, AssetFileInfo assetInfo,
        IDictionary<ushort, string?> monoScriptClassNameCache)
    {
        if (assetInfo.TypeId != (int)AssetClassID.MonoBehaviour)
        {
            return null;
        }

        ushort scriptIndex;
        try
        {
            scriptIndex = assetInfo.GetScriptIndex(assetsFile.file);
        }
        catch
        {
            return null;
        }

        if (scriptIndex == ushort.MaxValue)
        {
            return null;
        }

        if (monoScriptClassNameCache.TryGetValue(scriptIndex, out var cachedClassName))
        {
            return cachedClassName;
        }

        if (TryGetTypeTreeRootClassName(assetsFile, scriptIndex, out var typeTreeClassName))
        {
            monoScriptClassNameCache[scriptIndex] = typeTreeClassName;
            return typeTreeClassName;
        }

        AssetTypeReference? scriptInfo;
        try
        {
            scriptInfo = AssetHelper.GetAssetsFileScriptInfo(manager, assetsFile, scriptIndex);
        }
        catch
        {
            return null;
        }

        monoScriptClassNameCache[scriptIndex] = scriptInfo?.ClassName;
        return scriptInfo?.ClassName;
    }

    private static bool TryGetTypeTreeRootClassName(AssetsFileInstance assetsFile,
                                                    ushort scriptIndex, out string className)
    {
        className = string.Empty;

        var typeTreeType = assetsFile.file.Metadata.FindTypeTreeTypeByID(
            (int)AssetClassID.MonoBehaviour, scriptIndex);

        if (typeTreeType == null || typeTreeType.Nodes.Count == 0 ||
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

    private static string ReadStringField(AssetTypeValueField field)
    {
        if (field.IsDummy)
        {
            return string.Empty;
        }

        return field.AsString ?? string.Empty;
    }

    private static long GetPointerPathId(AssetTypeValueField pointerField)
    {
        if (pointerField.IsDummy)
        {
            return 0;
        }

        return pointerField[PointerPathIdFieldName].AsLong;
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

        return $"pathid_{assetInfo.PathId}";
    }

    private static string BuildOutputDirectory(string inputPath, string suffix)
    {
        var parentDirectory =
            Path.GetDirectoryName(inputPath) ?? Directory.GetCurrentDirectory();
        var stem = Path.GetFileNameWithoutExtension(inputPath);
        return Path.Combine(parentDirectory, $"{stem}{suffix}");
    }

    private static string GetUniqueOutputPath(string directory, string rawFileNameStem,
                                              string extension, ISet<string> usedNames)
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

        throw new InvalidOperationException(
            $"Failed to generate a unique output path for '{fileNameStem}'.");
    }

    private static string SanitizeFileName(string value)
    {
        var cleanedChars =
            value.Trim().Select(ch => InvalidFileNameChars.Contains(ch) ? '_' : ch).ToArray();

        return new string(cleanedChars);
    }

    private sealed record DumpResult(string DumpRootDirectory, string ProgramsDirectory,
                                     string VarsDirectory, int DumpedCount, int DumpedVarCount);
    private sealed record ProgramDumpInfo(long PathId, string OutputName);
    private sealed record UdonBehaviourInfo(long GameObjectId, long ProgramPathId,
                                            string SerializedPublicVariables);
    private sealed record ProgramVarMapEntry(long GameObjectId, string PublicVar);
}
