using System;
using System.Collections.Generic;
using System.IO;
using UnityEditor;
using UnityEngine;

[Serializable]
public class CompilerInputData
{
    public List<CompilationRequest> requests;
}

[Serializable]
public class CompilerOutputData
{
    public List<string> results;
    public string error;
}

[InitializeOnLoad]
public static class UdonCompilerCLI
{
    private const string STATE_KEY_CLI_ACTIVE = "UdonCompilerCLI_IsActive";
    private const string STATE_KEY_OUTPUT_PATH = "UdonCompilerCLI_OutputPath";

    static UdonCompilerCLI()
    {
        if (SessionState.GetBool(STATE_KEY_CLI_ACTIVE, false))
        {
            UdonSharpSourceTextCompiler.OnBatchDumpCompleted += OnBatchFinished;
        }
    }

    public static void Run()
    {
        try
        {
            Debug.Log("[CLI] Starting Udon Compilation CLI...");

            string inputPath = GetArg("-inputFile");
            string outputPath = GetArg("-outputFile");

            if (string.IsNullOrEmpty(inputPath) || string.IsNullOrEmpty(outputPath))
            {
                throw new Exception("Usage: -executeMethod UdonCompilerCLI.Run -inputFile <path> -outputFile <path>");
            }

            if (!File.Exists(inputPath))
            {
                throw new FileNotFoundException($"Input file not found: {inputPath}");
            }

            string jsonContent = File.ReadAllText(inputPath);
            CompilerInputData inputData = JsonUtility.FromJson<CompilerInputData>(jsonContent);

            if (inputData == null || inputData.requests == null || inputData.requests.Count == 0)
            {
                throw new Exception("Input JSON is invalid or empty.");
            }

            Debug.Log($"[CLI] Loaded {inputData.requests.Count} requests. Starting compilation...");

            SessionState.SetBool(STATE_KEY_CLI_ACTIVE, true);
            SessionState.SetString(STATE_KEY_OUTPUT_PATH, outputPath);

            UdonSharpSourceTextCompiler.CompileBatch(inputData.requests);
        }
        catch (Exception ex)
        {
            Debug.LogError($"[CLI] Fatal Error: {ex.Message}");
            EditorApplication.Exit(1);
        }
    }

    private static void OnBatchFinished(List<string> results)
    {
        Debug.Log("[CLI] Compilation finished. Writing results...");

        try
        {
            string outputPath = SessionState.GetString(STATE_KEY_OUTPUT_PATH, "output.json");

            CompilerOutputData outputData = new CompilerOutputData
            {
                results = results,
                error = null
            };

            string jsonResult = JsonUtility.ToJson(outputData, true);
            File.WriteAllText(outputPath, jsonResult);

            Debug.Log($"[CLI] Success! Output written to {outputPath}");

            SessionState.SetBool(STATE_KEY_CLI_ACTIVE, false);
            SessionState.EraseString(STATE_KEY_OUTPUT_PATH);

            EditorApplication.Exit(0);
        }
        catch (Exception ex)
        {
            Debug.LogError($"[CLI] Error writing output: {ex.Message}");
            EditorApplication.Exit(1);
        }
    }

    private static string GetArg(string name)
    {
        string[] args = Environment.GetCommandLineArgs();
        for (int i = 0; i < args.Length; i++)
        {
            if (args[i] == name && args.Length > i + 1)
            {
                return args[i + 1];
            }
        }
        return null;
    }
}
