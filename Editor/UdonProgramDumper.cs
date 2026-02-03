// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2025 ParaN3xus <paran3xus007@gmail.com>

using System;
using System.Collections.Generic;
using System.Linq;
using System.Runtime.CompilerServices;
using System.Text;
using Newtonsoft.Json;
using UnityEditor;
using UnityEngine;
using VRC.Udon.Common;
using VRC.Udon.Common.Interfaces;
using VRC.Udon.Serialization.OdinSerializer;

public static class UdonProgramDumper
{
    public static string DumpUdonProgram(IUdonProgram program)
    {
        var settings = new JsonSerializerSettings { Formatting = Formatting.Indented };

        var serialized = new UdonProgramSerialized
        {
            byteCodeHex = BitConverter.ToString(program.ByteCode).Replace("-", ""),
            byteCodeLength = program.ByteCode.Length,
            symbols = SerializeSymbolTable(program.SymbolTable, program.EntryPoints),
            entryPoints = SerializeEntryPoints(program.EntryPoints),
            heapInitialValues = SerializeHeap(program.Heap),
        };

        return JsonConvert.SerializeObject(serialized, settings);
    }

    static Dictionary<string, SymbolInfo> SerializeSymbolTable(
        IUdonSymbolTable symbolTable,
        IUdonSymbolTable entryPoints
    )
    {
        var result = new Dictionary<string, SymbolInfo>();

        foreach (var symbolName in symbolTable.GetSymbols())
        {
            uint address = symbolTable.GetAddressFromSymbol(symbolName);
            string type = symbolTable.GetSymbolType(symbolName).FullName;

            result[symbolName] = new SymbolInfo
            {
                name = symbolName,
                type = type,
                address = address,
            };
        }

        return result;
    }

    static List<EntryPointInfo> SerializeEntryPoints(IUdonSymbolTable entryPoints)
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

    static Dictionary<uint, HeapEntry> SerializeHeap(IUdonHeap heap)
    {
        var result = new Dictionary<uint, HeapEntry>();
        var heapDump = new List<ValueTuple<uint, IStrongBox, Type>>();
        heap.DumpHeapObjects(heapDump);

        Debug.Log("Multiple warnings may appear in the console, " +
                    "but as long as the dump does not terminate due to a fatal error, " +
                    "it is normal behavior.");

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
                type = type.FullName,
                value = wrappedValue,
            };
            result[address] = entry;
        }

        return result;
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
        public string type;
        public uint address;
    }

    [Serializable]
    public class HeapEntry
    {
        public uint address;
        public string type;
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

}
