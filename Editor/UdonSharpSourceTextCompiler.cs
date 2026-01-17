using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using UnityEditor;
using UnityEngine;
using UdonSharp;
using UdonSharp.Compiler;
using VRC.Udon.Common.Interfaces;

[Serializable]
public struct CompilationRequest
{
    public string sourceCode;
    public string className;

    public CompilationRequest(string source, string name)
    {
        sourceCode = source;
        className = name;
    }
}

public static class UdonSharpSourceTextCompiler
{
    private const string TEMP_FOLDER = "Assets/_USharpSourceTextCompiler";
    private const string STATE_KEY_COMPILING = "USharpSourceTextCompiler_IsCompiling";
    private const string STATE_KEY_FILENAMES = "USharpSourceTextCompiler_FileNames";

    public static event Action<string> OnDumpCompleted;
    public static event Action<List<string>> OnBatchDumpCompleted;

    public static void CompileBatch(List<CompilationRequest> requests)
    {
        if (requests == null || requests.Count == 0) return;

        if (AssetDatabase.IsValidFolder(TEMP_FOLDER))
        {
            AssetDatabase.DeleteAsset(TEMP_FOLDER);
        }
        AssetDatabase.CreateFolder("Assets", "_USharpSourceTextCompiler");

        List<string> fileNames = new List<string>();

        foreach (var req in requests)
        {
            string finalSource = req.sourceCode;
            string fileName = req.className;

            string filePath = $"{TEMP_FOLDER}/{fileName}.cs";
            File.WriteAllText(filePath, finalSource);

            fileNames.Add(fileName);
        }

        SessionState.SetBool(STATE_KEY_COMPILING, true);
        SessionState.SetString(STATE_KEY_FILENAMES, string.Join(";", fileNames));

        AssetDatabase.Refresh();

        Debug.Log($"[U# Source Text Compiler] Batch processing {requests.Count} files. Waiting for Domain Reload...");
    }

    public static void CompileAndDump(string sourceCode, string fileName)
    {
        var req = new CompilationRequest(sourceCode, fileName);
        CompileBatch(new List<CompilationRequest> { req });
    }

    [UnityEditor.Callbacks.DidReloadScripts]
    private static void OnScriptsReloaded()
    {
        if (!SessionState.GetBool(STATE_KEY_COMPILING, false)) return;
        SessionState.SetBool(STATE_KEY_COMPILING, false);

        EditorApplication.delayCall += ExecuteBatchCompilation;
    }

    private static void ExecuteBatchCompilation()
    {
        string joinedNames = SessionState.GetString(STATE_KEY_FILENAMES, "");
        if (string.IsNullOrEmpty(joinedNames)) return;

        string[] fileNames = joinedNames.Split(new[] { ';' }, StringSplitOptions.RemoveEmptyEntries);
        List<UdonSharpProgramAsset> assetsToCompile = new List<UdonSharpProgramAsset>();

        foreach (var fileName in fileNames)
        {
            string scriptPath = $"{TEMP_FOLDER}/{fileName}.cs";
            string assetPath = $"{TEMP_FOLDER}/{fileName}.asset";

            MonoScript script = AssetDatabase.LoadAssetAtPath<MonoScript>(scriptPath);
            if (script == null)
            {
                Debug.LogError($"[U# Source Text Compiler] Failed to load script: {fileName}");
                continue;
            }

            UdonSharpProgramAsset programAsset = AssetDatabase.LoadAssetAtPath<UdonSharpProgramAsset>(assetPath);
            if (programAsset == null)
            {
                programAsset = ScriptableObject.CreateInstance<UdonSharpProgramAsset>();
                AssetDatabase.CreateAsset(programAsset, assetPath);
            }

            if (programAsset.sourceCsScript != script)
            {
                programAsset.sourceCsScript = script;
                EditorUtility.SetDirty(programAsset);
            }

            assetsToCompile.Add(programAsset);
        }

        AssetDatabase.SaveAssets();

        UdonSharpCompilerV1.CompileSync();

        List<string> jsonResults = new List<string>();

        foreach (var programAsset in assetsToCompile)
        {
            IUdonProgram program = programAsset.GetRealProgram();
            if (program != null)
            {
                string json = UdonProgramDumper.DumpUdonProgram(program);
                jsonResults.Add(json);
            }
            else
            {
                jsonResults.Add("ERROR: Compile Failed");
                Debug.LogError($"[U# Source Text Compiler] Failed to compile {programAsset.name}");
            }
        }

        Debug.Log($"[U# Source Text Compiler] Batch Dump Completed. Count: {jsonResults.Count}");

        OnBatchDumpCompleted?.Invoke(jsonResults);

        if (jsonResults.Count == 1)
        {
            OnDumpCompleted?.Invoke(jsonResults[0]);
        }
    }
}
