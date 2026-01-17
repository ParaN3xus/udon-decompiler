// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2025 ParaN3xus <paran3xus007@gmail.com>

using System;
using System.IO;
using System.IO.Compression;
using System.Text;
using System.Text.RegularExpressions;
using UnityEditor;
using UnityEngine;
using VRC.Udon.Common;
using VRC.Udon.Serialization.OdinSerializer;

public class UdonProgramDumperGUI : EditorWindow
{
    private string folderPath = "";

    [MenuItem("Tools/Udon Program Dumper")]
    public static void ShowWindow()
    {
        GetWindow<UdonProgramDumperGUI>("Udon Program Dumper");
    }

    void OnGUI()
    {
        GUILayout.Label("Udon Program Dumper", EditorStyles.boldLabel);
        folderPath = EditorGUILayout.TextField("Folder Path:", folderPath);

        if (GUILayout.Button("Dump All .asset Files"))
        {
            DumpAllFiles();
        }
    }

    void DumpAllFiles()
    {
        if (!Directory.Exists(folderPath))
        {
            Debug.LogError($"Folder not found: {folderPath}");
            return;
        }

        string[] assetFiles = Directory.GetFiles(folderPath, "*.asset", SearchOption.TopDirectoryOnly);
        if (assetFiles.Length == 0)
        {
            Debug.LogWarning($"No .asset files found in: {folderPath}");
            return;
        }

        foreach (string filePath in assetFiles)
        {
            try
            {
                byte[] decompressedData = ExtractAndDecompressFromAsset(filePath);
                if (decompressedData == null) continue;

                using var memoryStream = new MemoryStream(decompressedData);
                var reader = new BinaryDataReader(memoryStream, new DeserializationContext());
                UdonProgram program = VRC.Udon.Serialization.OdinSerializer.SerializationUtility.DeserializeValue<UdonProgram>(reader);

                if (program != null)
                {
                    SaveDumpedJson(filePath, program);
                }
            }
            catch (Exception e)
            {
                Debug.LogError($"✗ {Path.GetFileName(filePath)} - Error: {e.Message}");
            }
        }

        Debug.Log("Dumped!");
        AssetDatabase.Refresh();
    }

    void SaveDumpedJson(string originalFilePath, UdonProgram program)
    {
        string json = UdonProgramDumper.DumpUdonProgram(program);

        string outputDir = Path.Combine(Path.GetDirectoryName(originalFilePath), "dumped");
        if (!Directory.Exists(outputDir)) Directory.CreateDirectory(outputDir);

        string outputFilePath = Path.Combine(outputDir, $"{Path.GetFileName(originalFilePath)}.json");
        File.WriteAllText(outputFilePath, json, new UTF8Encoding(false));
        Debug.Log($"✓ Generated: {outputFilePath}");
    }

    static byte[] ExtractAndDecompressFromAsset(string assetPath)
    {
        string yamlContent = File.ReadAllText(assetPath, Encoding.UTF8);
        var match = Regex.Match(yamlContent, @"serializedProgramCompressedBytes:\s*([0-9a-fA-F]+)");

        if (!match.Success) return null;

        byte[] compressedBytes = HexStringToByteArray(match.Groups[1].Value.Trim());
        return DecompressGZip(compressedBytes);
    }

    static byte[] HexStringToByteArray(string hex)
    {
        byte[] bytes = new byte[hex.Length / 2];
        for (int i = 0; i < hex.Length; i += 2)
            bytes[i / 2] = Convert.ToByte(hex.Substring(i, 2), 16);
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
}
