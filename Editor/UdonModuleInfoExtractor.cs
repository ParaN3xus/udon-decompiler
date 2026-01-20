// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2025 ParaN3xus <paran3xus007@gmail.com>

using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Text;
using Newtonsoft.Json;
using UnityEditor;
using UnityEngine;
using VRC.Udon.Common.Interfaces;
using VRC.Udon.Security;
using VRC.Udon.Wrapper;
using VRC.Udon.Graph;
using VRC.Udon.Graph.Interfaces;
using VRC.Udon.Graph.NodeRegistries;

public class UdonModuleInfoExtractor : EditorWindow
{
    [Serializable]
    public class ModuleDefinition
    {
        public string type;
        public List<FunctionDefinition> functions;
    }

#nullable enable
    [Serializable]
    public class FunctionDefinition
    {
        public string name = null!;
        public int parameterCount;

        [JsonProperty(NullValueHandling = NullValueHandling.Ignore)]
        public string? originalName;
        public string defType = null!;

        [JsonProperty(NullValueHandling = NullValueHandling.Ignore)]
        public bool? isStatic;

        [JsonProperty(NullValueHandling = NullValueHandling.Ignore)]
        public bool? returnsVoid;
    }
#nullable restore

    private class NodeInfoCache
    {
        public NodeRegistryUtilities.DefinitionType DefType;
        public MemberInfo Member;
    }

    [MenuItem("Tools/Extract Udon Module Info")]
    public static void ExtractModuleInfo()
    {
        var registryLookup = BuildRegistryLookup();
        Debug.Log($"Registry lookup built with {registryLookup.Count} entries.");

        UnityEngineObjectSecurityBlacklist blacklist = new UnityEngineObjectSecurityBlacklist();
        UdonDefaultWrapperFactory udonWrapperFactory = new UdonDefaultWrapperFactory(blacklist);
        IUdonWrapper udonWrapper = udonWrapperFactory.GetWrapper();
        List<Type> moduleTypes = GetWrapperModuleTypes(udonWrapperFactory);

        var result = new Dictionary<string, ModuleDefinition>();

        foreach (Type moduleType in moduleTypes)
        {
            try
            {
                object instance = Activator.CreateInstance(moduleType, new object[] { udonWrapper, blacklist });

                PropertyInfo nameProperty = moduleType.GetProperty("Name", BindingFlags.Public | BindingFlags.Instance);
                string moduleName = nameProperty?.GetValue(instance) as string;

                if (string.IsNullOrEmpty(moduleName)) continue;

                FieldInfo parameterCountsField = moduleType.GetField("_parameterCounts", BindingFlags.NonPublic | BindingFlags.Instance);
                object rawCounts = parameterCountsField?.GetValue(instance);
                Dictionary<string, int> paramCounts = null;

                if (rawCounts is Lazy<Dictionary<string, int>> lazyCounts) paramCounts = lazyCounts.Value;
                else if (rawCounts is Dictionary<string, int> directCounts) paramCounts = directCounts;

                if (paramCounts == null) continue;

                ModuleDefinition moduleDef = new ModuleDefinition
                {
                    functions = new List<FunctionDefinition>(),
                    type = null
                };

                foreach (var kvp in paramCounts)
                {
                    string funcName = kvp.Key;
                    int pCount = kvp.Value;
                    string fullNodeName = $"{moduleName}.{funcName}";

                    FunctionDefinition funcDef = new FunctionDefinition
                    {
                        name = funcName,
                        parameterCount = pCount,
                        originalName = funcName,
                        defType = "UNKNOWN",
                        isStatic = null
                    };

                    if (registryLookup.TryGetValue(fullNodeName, out NodeInfoCache cacheInfo))
                    {
                        if (moduleDef.type == null && cacheInfo.Member.DeclaringType != null)
                        {
                            moduleDef.type = cacheInfo.Member.DeclaringType.FullName;
                        }

                        if (cacheInfo.DefType == NodeRegistryUtilities.DefinitionType.OPERATOR)
                            funcDef.originalName = null;

                        funcDef.defType = cacheInfo.DefType.ToString();
                        if (cacheInfo.Member != null)
                        {
                            if (cacheInfo.DefType == NodeRegistryUtilities.DefinitionType.CTOR_INFO)
                                funcDef.originalName = null;
                            else
                                funcDef.originalName = cacheInfo.Member.Name;

                            if (cacheInfo.DefType == NodeRegistryUtilities.DefinitionType.METHOD_INFO && cacheInfo.Member is MethodInfo mi)
                            {
                                funcDef.returnsVoid = mi.ReturnType == typeof(void);
                                funcDef.isStatic = mi.IsStatic;
                            }
                            else if (cacheInfo.DefType == NodeRegistryUtilities.DefinitionType.CTOR_INFO && cacheInfo.Member is ConstructorInfo ci)
                                funcDef.isStatic = ci.IsStatic;
                        }
                    }

                    moduleDef.functions.Add(funcDef);
                }

                result[moduleName] = moduleDef;
            }
            catch (Exception ex)
            {
                Debug.LogWarning($"Error processing module {moduleType.Name}: {ex.Message}");
            }
        }

        FixResult(result);

        var settings = new JsonSerializerSettings { NullValueHandling = NullValueHandling.Ignore, Formatting = Formatting.Indented };
        string json = JsonConvert.SerializeObject(result, settings);
        string path = Path.Combine(Application.dataPath, "UdonModuleInfo.json");
        File.WriteAllText(path, json, new UTF8Encoding(false));

        AssetDatabase.Refresh();
        Debug.Log($"Module info saved to: {path}");
        Debug.Log($"Total modules extracted: {result.Count}");
    }

    private static void FixResult(Dictionary<string, ModuleDefinition> result)
    {
        foreach (var module in result.Values)
        {
            if (module.functions == null) continue;

            foreach (var func in module.functions)
            {
                if (func.name.StartsWith("__op_"))
                {
                    func.defType = "OPERATOR";
                    func.originalName = null;
                    func.isStatic = null;
                    func.returnsVoid = null;
                }
                else if (func.name.StartsWith("__ctor__"))
                {
                    func.defType = "CTOR_INFO";
                    func.originalName = null;
                    func.isStatic = null;
                    func.returnsVoid = null;
                }
                else if (func.name.StartsWith("__get_") || func.name.StartsWith("__set_"))
                {
                    func.defType = "FIELD_INFO";
                    func.isStatic = null;
                    func.returnsVoid = null;

                    if (func.name.Length > 6)
                    {
                        string withoutPrefix = func.name.Substring(6);
                        int splitIndex = withoutPrefix.IndexOf("__");
                        if (splitIndex > 0)
                        {
                            func.originalName = withoutPrefix.Substring(0, splitIndex);
                        }
                        else
                        {
                            func.originalName = withoutPrefix;
                        }
                    }
                }
                else
                {
                    if (func.defType == "OPERATOR" ||
                        func.defType == "CTOR_INFO" ||
                        func.defType == "FIELD_INFO")
                    {
                        func.defType = "METHOD_INFO";
                        func.isStatic = true;
                        func.returnsVoid = true;
                        func.originalName = func.name;
                    }
                }
            }
        }
    }

    private static Dictionary<string, NodeInfoCache> BuildRegistryLookup()
    {
        var lookup = new Dictionary<string, NodeInfoCache>();
        RootNodeRegistry rootRegistry = new RootNodeRegistry();

        PropertyInfo nextRegistriesProp = typeof(RootNodeRegistry).GetProperty("NextRegistries", BindingFlags.NonPublic | BindingFlags.Instance);
        if (nextRegistriesProp == null) return lookup;

        var nextRegistries = nextRegistriesProp.GetValue(rootRegistry) as Dictionary<string, INodeRegistry>;
        if (nextRegistries == null) return lookup;

        foreach (var registry in nextRegistries.Values)
        {
            PropertyInfo nodeDefsProp = registry.GetType().GetProperty("NodeDefinitions", BindingFlags.NonPublic | BindingFlags.Instance | BindingFlags.Public);
            if (nodeDefsProp == null) continue;

            var definitions = nodeDefsProp.GetValue(registry) as Dictionary<string, UdonNodeDefinition>;
            if (definitions == null) continue;

            foreach (var def in definitions.Values)
            {
                try
                {
                    var info = NodeRegistryUtilities.GetNodeDefinitionInfo(def);
                    var cache = new NodeInfoCache { DefType = info.definitionType };

                    switch (info.definitionType)
                    {
                        case NodeRegistryUtilities.DefinitionType.METHOD_INFO:
                            cache.Member = info.info as MethodInfo;
                            break;
                        case NodeRegistryUtilities.DefinitionType.FIELD_INFO:
                            cache.Member = info.info as FieldInfo;
                            break;
                        case NodeRegistryUtilities.DefinitionType.CTOR_INFO:
                            cache.Member = info.info as ConstructorInfo;
                            break;
                    }

                    if (!lookup.ContainsKey(def.fullName))
                    {
                        lookup[def.fullName] = cache;
                    }
                }
                catch
                {
                    Debug.LogWarning($"Failed to get node definition info of {def.fullName}");
                }
            }
        }
        Debug.LogWarning($"At this stage, a small number of errors are acceptable");
        return lookup;
    }

    private static List<Type> GetWrapperModuleTypes(UdonDefaultWrapperFactory udonWrapperFactory)
    {
        if (udonWrapperFactory == null)
        {
            Debug.LogWarning("Given factory is null!");
            return new List<Type>();
        }

        FieldInfo field = typeof(UdonDefaultWrapperFactory).GetField(
            "_wrapperModuleTypes",
            BindingFlags.NonPublic | BindingFlags.Instance
        );

        if (field == null)
        {
            Debug.LogError("Cannot find _wrapperModuleTypes field");
            return new List<Type>();
        }

        var hashSet = field.GetValue(udonWrapperFactory) as HashSet<Type>;

        return hashSet != null ? new List<Type>(hashSet) : new List<Type>();
    }
}
