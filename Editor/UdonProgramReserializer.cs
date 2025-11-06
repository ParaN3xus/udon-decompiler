// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2025 ParaN3xus <paran3xus007@gmail.com>

using System;
using System.Collections.Generic;
using System.IO;
using System.IO.Compression;
using System.Linq;
using System.Runtime.CompilerServices;
using System.Text;
using System.Text.RegularExpressions;
using Newtonsoft.Json;
using UnityEditor;
using UnityEngine;
using VRC.Udon.Common;
using VRC.Udon.Common.Interfaces;
using VRC.Udon.Serialization.OdinSerializer;

public class UdonProgramReserializer : EditorWindow
{
    private string folderPath = "";

    [MenuItem("Tools/Udon Program Reserializer")]
    public static void ShowWindow()
    {
        GetWindow<UdonProgramReserializer>("Udon Reserializer");
    }

    void OnGUI()
    {
        GUILayout.Label("Udon Program Reserializer", EditorStyles.boldLabel);

        folderPath = EditorGUILayout.TextField("Folder Path:", folderPath);

        if (GUILayout.Button("Reserialize All .asset Files"))
        {
            ReserializeAllFiles();
        }
    }

    void ReserializeAllFiles()
    {
        if (!Directory.Exists(folderPath))
        {
            Debug.LogError($"Folder not found: {folderPath}");
            return;
        }

        string[] assetFiles = Directory.GetFiles(
            folderPath,
            "*.asset",
            SearchOption.TopDirectoryOnly
        );

        if (assetFiles.Length == 0)
        {
            Debug.LogWarning($"No .asset files found in: {folderPath}");
            return;
        }

        Debug.Log($"Found {assetFiles.Length} .asset files");

        foreach (string filePath in assetFiles)
        {
            try
            {
                byte[] decompressedData = ExtractAndDecompressFromAsset(filePath);

                if (decompressedData == null)
                {
                    Debug.LogWarning($"No valid serializedProgramCompressedBytes found");
                    continue;
                }

                using var memoryStream = new MemoryStream(decompressedData);
                var context = new DeserializationContext();
                var reader = new BinaryDataReader(memoryStream, context);
                UdonProgram program =
                    VRC.Udon.Serialization.OdinSerializer.SerializationUtility.DeserializeValue<UdonProgram>(
                        reader
                    );

                if (program != null)
                {
                    GenerateSerializedFile(filePath, program);
                }
                else
                {
                    Debug.LogWarning(
                        $"Failed to reserialize {Path.GetFileName(filePath)}: Program is null"
                    );
                }
            }
            catch (Exception e)
            {
                Debug.LogError(
                    $"✗ {Path.GetFileName(filePath)} - Error: {e.Message}\n{e.StackTrace}"
                );
            }
        }

        Debug.Log("Reserialize complete!");
        AssetDatabase.Refresh();
    }

    static byte[] ExtractAndDecompressFromAsset(string assetPath)
    {
        string yamlContent = File.ReadAllText(assetPath, Encoding.UTF8);

        var match = Regex.Match(
            yamlContent,
            @"serializedProgramCompressedBytes:\s*([0-9a-fA-F]+)",
            RegexOptions.Multiline
        );

        if (!match.Success)
        {
            return null;
        }

        string hexString = match.Groups[1].Value.Trim();

        byte[] compressedBytes = HexStringToByteArray(hexString);

        return DecompressGZip(compressedBytes);
    }

    static byte[] HexStringToByteArray(string hex)
    {
        int length = hex.Length;
        byte[] bytes = new byte[length / 2];

        for (int i = 0; i < length; i += 2)
        {
            bytes[i / 2] = Convert.ToByte(hex.Substring(i, 2), 16);
        }

        return bytes;
    }

    static byte[] DecompressGZip(byte[] compressedData)
    {
        using var compressedStream = new MemoryStream(compressedData);
        using var gzipStream = new GZipStream(compressedStream, CompressionMode.Decompress);
        using var decompressedStream = new MemoryStream();

        gzipStream.CopyTo(decompressedStream);
        return decompressedStream.ToArray();
    }

    void GenerateSerializedFile(string originalFilePath, IUdonProgram program)
    {
        var settings = new JsonSerializerSettings { Formatting = Formatting.Indented };

        string fileName = Path.GetFileName(originalFilePath);
        string outputDir = Path.Combine(Path.GetDirectoryName(originalFilePath), "serialized");
        if (!Directory.Exists(outputDir))
        {
            Directory.CreateDirectory(outputDir);
        }

        var serialized = new UdonProgramSerialized
        {
            byteCodeHex = BitConverter.ToString(program.ByteCode).Replace("-", ""),
            byteCodeLength = program.ByteCode.Length,
            symbols = SerializeSymbolTable(program.SymbolTable, program.EntryPoints),
            entryPoints = SerializeEntryPoints(program.EntryPoints),
            heapInitialValues = SerializeHeap(program.Heap),
        };

        string json = JsonConvert.SerializeObject(serialized, settings);

        string outputFilePath = Path.Combine(outputDir, $"{fileName}.json");
        File.WriteAllText(outputFilePath, json, new UTF8Encoding(false));
        Debug.Log($"✓ Generated: {outputFilePath}");
    }

    Dictionary<string, SymbolInfo> SerializeSymbolTable(
        IUdonSymbolTable symbolTable,
        IUdonSymbolTable entryPoints
    )
    {
        var result = new Dictionary<string, SymbolInfo>();

        foreach (var symbolName in symbolTable.GetSymbols())
        {
            uint address = symbolTable.GetAddressFromSymbol(symbolName);
            Type type = symbolTable.GetSymbolType(symbolName);

            result[symbolName] = new SymbolInfo
            {
                name = symbolName,
                type = type,
                address = address,
            };
        }

        return result;
    }

    List<EntryPointInfo> SerializeEntryPoints(IUdonSymbolTable entryPoints)
    {
        return entryPoints
            .GetExportedSymbols()
            .Select(name => new EntryPointInfo
            {
                name = name,
                address = entryPoints.GetAddressFromSymbol(name),
            })
            .ToList();
    }

    Dictionary<uint, HeapEntry> SerializeHeap(IUdonHeap heap)
    {
        var result = new Dictionary<uint, HeapEntry>();
        var heapDump = new List<ValueTuple<uint, IStrongBox, Type>>();
        heap.DumpHeapObjects(heapDump);

        foreach (var (address, strongBox, type) in heapDump)
        {
            object rawValue = strongBox?.Value;
            HeapEntryValue wrappedValue;

            try
            {
                JsonConvert.SerializeObject(rawValue);

                wrappedValue = new HeapEntryValue { isSerializable = true, value = rawValue };
            }
            catch (Exception)
            {
                wrappedValue = new HeapEntryValue
                {
                    isSerializable = false,
                    value = new { type = type?.FullName, toString = rawValue?.ToString() },
                };
            }
            var entry = new HeapEntry
            {
                address = address,
                type = type,
                value = wrappedValue,
            };
            result[address] = entry;
        }

        return result;
    }
}

[Serializable]
public class UdonProgramSerialized
{
    public string byteCodeHex;
    public int byteCodeLength;
    public Dictionary<string, SymbolInfo> symbols;
    public List<EntryPointInfo> entryPoints;
    public Dictionary<uint, HeapEntry> heapInitialValues;
}

[Serializable]
public class SymbolInfo
{
    public string name;
    public Type type;
    public uint address;
}

[Serializable]
public class HeapEntry
{
    public uint address;
    public Type type;
    public HeapEntryValue value;
}

[Serializable]
public class HeapEntryValue
{
    public bool isSerializable;
    public object value;
}

[Serializable]
public class EntryPointInfo
{
    public string name;
    public uint address;
}
