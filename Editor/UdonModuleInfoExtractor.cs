// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2025 ParaN3xus <paran3xus007@gmail.com>

using System;
using System.Collections.Generic;
using System.IO;
using System.Reflection;
using System.Text;
using Newtonsoft.Json;
using UnityEditor;
using UnityEngine;
using VRC.Udon.Common.Attributes;
using VRC.Udon.Common.Interfaces;
using VRC.Udon.Security.Interfaces;
using VRC.Udon.Security;
using VRC.Udon.Wrapper;

public class UdonModuleInfoExtractor : EditorWindow
{
    [MenuItem("Tools/Extract Udon Module Info")]
    public static void ExtractModuleInfo()
    {
        var result = new Dictionary<string, Dictionary<string, int>>();

        UnityEngineObjectSecurityBlacklist blacklist = new UnityEngineObjectSecurityBlacklist();
        UdonDefaultWrapperFactory udonWrapperFactory = new UdonDefaultWrapperFactory(blacklist);
        IUdonWrapper udonWrapper = udonWrapperFactory.GetWrapper();

        List<Type> moduleTypes = GetWrapperModuleTypesFromAttribute();

        foreach (Type moduleType in moduleTypes)
        {
            try
            {
                object instance = null;

                instance = Activator.CreateInstance(
                    moduleType,
                    new object[] { udonWrapper, blacklist }
                );

                PropertyInfo nameProperty = moduleType.GetProperty(
                    "Name",
                    BindingFlags.Public | BindingFlags.Instance
                );
                string name = nameProperty?.GetValue(instance) as string;

                if (string.IsNullOrEmpty(name))
                {
                    Debug.LogWarning($"Module {moduleType.Name} has no valid Name property");
                    continue;
                }

                FieldInfo parameterCountsField = moduleType.GetField(
                    "_parameterCounts",
                    BindingFlags.NonPublic | BindingFlags.Instance
                );

                if (
                    parameterCountsField?.GetValue(instance)
                    is Dictionary<string, int> parameterCounts
                )
                {
                    result[name] = new Dictionary<string, int>(parameterCounts);
                }
                else
                {
                    Debug.LogWarning($"Module {name} has no _parameterCounts field");
                }
            }
            catch (Exception ex)
            {
                Debug.LogWarning($"Failed to extract from {moduleType.Name}: {ex.Message}");
            }
        }

        string json = JsonConvert.SerializeObject(result, Formatting.Indented);

        string path = Path.Combine(Application.dataPath, "UdonModuleInfo.json");
        File.WriteAllText(path, json, new UTF8Encoding(false));

        AssetDatabase.Refresh();
        Debug.Log($"Module info saved to: {path}");
        Debug.Log($"Total modules extracted: {result.Count}");
    }

    private static List<Type> GetWrapperModuleTypesFromAttribute()
    {
        Type factoryType = typeof(UdonDefaultWrapperFactory);

        MethodInfo method = factoryType.GetMethod(
            "GetWrapperModuleTypesFromAttribute",
            BindingFlags.NonPublic | BindingFlags.Static
        );

        if (method == null)
        {
            Debug.LogError("Cannot find GetWrapperModuleTypesFromAttribute method");
            return new List<Type>();
        }

        return method.Invoke(null, null) as List<Type> ?? new List<Type>();
    }
}
