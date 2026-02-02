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

    [Serializable]
    public class FunctionDefinition
    {
        public string name;
        public List<string> parameters;
        public string originalName;
        public string defType;
        public bool isStatic;
        public bool returnsVoid;
    }


    [MenuItem("Tools/Extract Udon Module Info")]
    public static void ExtractModuleInfo()
    {
        var registryLookup = BuildRegistryLookup();
        Debug.Log($"Registry lookup built with {registryLookup.Count} entries.");

        UnityEngineObjectSecurityBlacklist blacklist = new UnityEngineObjectSecurityBlacklist();
        UdonDefaultWrapperFactory udonWrapperFactory = new UdonDefaultWrapperFactory(blacklist);
        IUdonWrapper udonWrapper = udonWrapperFactory.GetWrapper();
        List<Type> moduleWrapperTypes = GetWrapperModuleTypes(udonWrapperFactory);

        var result = new Dictionary<string, ModuleDefinition>();

        foreach (Type moduleWrapperType in moduleWrapperTypes)
        {
            object instance = Activator.CreateInstance(moduleWrapperType, new object[] { udonWrapper, blacklist });

            PropertyInfo nameProperty = moduleWrapperType.GetProperty("Name", BindingFlags.Public | BindingFlags.Instance);
            string moduleName = nameProperty?.GetValue(instance) as string;

            FieldInfo parameterCountsField = moduleWrapperType.GetField("_parameterCounts", BindingFlags.NonPublic | BindingFlags.Instance);
            Lazy<Dictionary<string, int>> rawCounts = parameterCountsField?.GetValue(instance) as Lazy<Dictionary<string, int>>;
            Dictionary<string, int> paramCounts = rawCounts.Value;

            Type moduleType = registryLookup[$"{moduleName}.{paramCounts.Keys.First()}"].type;
            ModuleDefinition moduleDef = new ModuleDefinition
            {
                functions = new List<FunctionDefinition>(paramCounts.Count),
                type = moduleType.FullName
            };

            foreach (var funcName in paramCounts.Keys)
            {
                string fullNodeName = $"{moduleName}.{funcName}";
                var udonNodeDef = registryLookup[fullNodeName];
                var parameters = udonNodeDef.parameters
                    .Select(param => param.parameterType.ToString())
                    .ToList();

                string defType = funcName.StartsWith("__op_") ? "op" :
                            funcName.StartsWith("__ctor__") ? "ctor" :
                            // todo: dictionary get/set has 3 params 
                            ((funcName.StartsWith("__get_") || funcName.StartsWith("__set_")) && parameters.Count >= 1 && parameters.Count <= 2) ? "prop" : "method";


                string originalName = null;
                var isStatic = false;
                var returnsVoid = false;

                int index = udonNodeDef.name.LastIndexOf(' ');
                if (defType == "prop")
                {
                    // __get_ or __set_
                    originalName = udonNodeDef.name[(index + 5)..];
                }
                if (defType == "method")
                {
                    originalName = udonNodeDef.name[(index + 1)..];
                }
                if ((defType == "prop" || defType == "method") && udonNodeDef.parameters.Count > 0)
                {
                    isStatic = !(udonNodeDef.parameters.First().name == "instance" && udonNodeDef.parameters[0].type == moduleType);
                    returnsVoid = udonNodeDef.parameters.Last().parameterType != UdonNodeParameter.ParameterType.OUT;
                }

                var func = new FunctionDefinition
                {
                    name = funcName,
                    originalName = originalName,
                    parameters = parameters,
                    isStatic = isStatic,
                    returnsVoid = returnsVoid,
                    defType = defType
                };
                moduleDef.functions.Add(func);
            }
            result[moduleName] = moduleDef;
        }

        var settings = new JsonSerializerSettings { Formatting = Formatting.Indented };
        string json = JsonConvert.SerializeObject(result, settings);
        string path = Path.Combine(Application.dataPath, "UdonModuleInfo.json");
        File.WriteAllText(path, json, new UTF8Encoding(false));

        AssetDatabase.Refresh();
        Debug.Log($"Module info saved to: {path}");
        Debug.Log($"Total modules extracted: {result.Count}");
    }

    private static Dictionary<string, UdonNodeDefinition> BuildRegistryLookup()
    {
        var lookup = new Dictionary<string, UdonNodeDefinition>();
        RootNodeRegistry rootRegistry = new RootNodeRegistry();
        PropertyInfo nextRegistriesProp = typeof(RootNodeRegistry).GetProperty("NextRegistries", BindingFlags.NonPublic | BindingFlags.Instance);
        var nextRegistries = nextRegistriesProp.GetValue(rootRegistry) as Dictionary<string, INodeRegistry>;

        foreach (var registry in nextRegistries.Values)
        {
            PropertyInfo nodeDefsProp = registry.GetType().GetProperty("NodeDefinitions", BindingFlags.NonPublic | BindingFlags.Instance | BindingFlags.Public);
            var definitions = nodeDefsProp.GetValue(registry) as Dictionary<string, UdonNodeDefinition>;
            foreach (var kvp in definitions)
            {
                lookup[kvp.Key] = kvp.Value;
            }
        }
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
