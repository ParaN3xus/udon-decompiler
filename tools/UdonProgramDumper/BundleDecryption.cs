using System.Buffers.Binary;
using System.Security.Cryptography;
using System.Text;

internal static partial class Program
{
    private const string DecryptionKeyMagic =
        "-MKEf6MXxgp5zN4YZkSdoj3oVygj6rLDTLXk6BQHnpyCRdv6R7d8Hm4nGs6LwQ9Ce";
    private const int EncryptedBlockUnitSize = 65_536 + 32;

    [Flags]
    private enum ArchiveFlags : uint
    {
        CompressionTypeMask = 0x3f,
        BlocksAndDirectoryInfoCombined = 0x40,
        BlocksInfoAtTheEnd = 0x80,
        OldWebPluginCompatibility = 0x100,
        BlockInfoNeedPaddingAtStart = 0x200,
        UnityCnEncryption = 0x400,
        UnityCnEncryption2 = 0x1000,
    }

    private static DecryptResult DecryptBundles(DecryptOptions options)
    {
        var keyBytes = ResolveDecryptKey(options);
        var fullInputPath = ResolveDecryptInputPath(options);
        var outputPath = ResolveDecryptOutputPath(fullInputPath, options.OutputPath);
        var fullOutputPath = Path.GetFullPath(outputPath);
        DecryptBundleFile(fullInputPath, fullOutputPath, keyBytes);
        return new DecryptResult(fullInputPath, fullOutputPath);
    }

    private static byte[] ResolveDecryptKey(DecryptOptions options)
    {
        if (string.IsNullOrWhiteSpace(options.Bpid))
        {
            throw new InvalidOperationException("--bpid is required.");
        }

        return SHA256.HashData(
            Encoding.UTF8.GetBytes(string.Concat(options.Bpid, DecryptionKeyMagic)));
    }

    private static string ResolveDecryptInputPath(DecryptOptions options)
    {
        var inputPath = options.InputPath ?? options.PositionalInputPath;
        if (string.IsNullOrWhiteSpace(inputPath))
        {
            throw new InvalidOperationException("An input file is required.");
        }

        var fullInputPath = Path.GetFullPath(inputPath);
        if (!File.Exists(fullInputPath))
        {
            throw new InvalidOperationException($"Input file does not exist: {fullInputPath}");
        }

        return fullInputPath;
    }

    private static void DecryptBundleFile(string inputPath, string outputPath, byte[] keyBytes)
    {
        using var input = File.OpenRead(inputPath);
        var header = ReadBundleHeader(input);

        if (!string.Equals(header.Signature, "UnityFS", StringComparison.Ordinal))
        {
            throw new InvalidOperationException(
                $"Unsupported bundle signature '{header.Signature}'.");
        }

        if ((header.Flags &
             (ArchiveFlags.UnityCnEncryption | ArchiveFlags.UnityCnEncryption2)) == 0)
        {
            throw new InvalidOperationException("The bundle is not marked as encrypted.");
        }

        var blocksInfoBytes = ReadBlocksInfoBytes(input, header);
        var blocksInfo = ParseBlocksInfo(blocksInfoBytes);
        var blockDataStart = GetBlockDataStartOffset(input.Length, header);
        var blockDataEnd = GetBlockDataEndOffset(input.Length, header);
        var blockJobs = BuildBlockJobs(blocksInfo.Blocks, blockDataStart, blockDataEnd);
        var decryptedBlocks = DecryptBlocksInParallel(inputPath, blockJobs, keyBytes, blocksInfo);

        var rebuiltBlocksInfoBytes = BuildBlocksInfoBytes(blocksInfo);
        var decryptedFlags =
            header.Flags & ~(ArchiveFlags.UnityCnEncryption | ArchiveFlags.UnityCnEncryption2);
        var outputLength =
            CalculateOutputLength(header, rebuiltBlocksInfoBytes.Length, decryptedBlocks);

        using var output = File.Create(outputPath);
        WriteBundleHeader(output, header with {
            Size = outputLength,
            CompressedBlocksInfoSize = (uint)rebuiltBlocksInfoBytes.Length,
            UncompressedBlocksInfoSize = (uint)rebuiltBlocksInfoBytes.Length,
            Flags = decryptedFlags,
        });

        AlignStream(output, header.Version >= 7 ? 16 : 1);

        if ((decryptedFlags & ArchiveFlags.BlocksInfoAtTheEnd) == 0)
        {
            output.Write(rebuiltBlocksInfoBytes);
            if ((decryptedFlags & ArchiveFlags.BlockInfoNeedPaddingAtStart) != 0)
            {
                AlignStream(output, 16);
            }
        }

        foreach (var blockBytes in decryptedBlocks)
        {
            output.Write(blockBytes);
        }

        if ((decryptedFlags & ArchiveFlags.BlocksInfoAtTheEnd) != 0)
        {
            output.Write(rebuiltBlocksInfoBytes);
        }
    }

    private static BundleHeader ReadBundleHeader(Stream stream)
    {
        var signature = ReadNullTerminatedString(stream, 20);
        var version = ReadUInt32BigEndian(stream);
        var unityVersion = ReadNullTerminatedString(stream);
        var unityRevision = ReadNullTerminatedString(stream);
        var size = ReadInt64BigEndian(stream);
        var compressedBlocksInfoSize = ReadUInt32BigEndian(stream);
        var uncompressedBlocksInfoSize = ReadUInt32BigEndian(stream);
        var flags = (ArchiveFlags)ReadUInt32BigEndian(stream);

        return new BundleHeader(signature, version, unityVersion, unityRevision, size,
                                compressedBlocksInfoSize, uncompressedBlocksInfoSize, flags);
    }

    private static void WriteBundleHeader(Stream stream, BundleHeader header)
    {
        WriteNullTerminatedString(stream, header.Signature);
        WriteUInt32BigEndian(stream, header.Version);
        WriteNullTerminatedString(stream, header.UnityVersion);
        WriteNullTerminatedString(stream, header.UnityRevision);
        WriteInt64BigEndian(stream, header.Size);
        WriteUInt32BigEndian(stream, header.CompressedBlocksInfoSize);
        WriteUInt32BigEndian(stream, header.UncompressedBlocksInfoSize);
        WriteUInt32BigEndian(stream, (uint)header.Flags);
    }

    private static byte[] ReadBlocksInfoBytes(Stream stream, BundleHeader header)
    {
        if (header.CompressedBlocksInfoSize != header.UncompressedBlocksInfoSize)
        {
            throw new InvalidOperationException(
                $"Unsupported blocks info compression layout: 0x{header.Flags:X}.");
        }

        var blocksInfoOffset = (header.Flags & ArchiveFlags.BlocksInfoAtTheEnd) != 0
                                   ? stream.Length - header.CompressedBlocksInfoSize
                                   : AlignValue(stream.Position, header.Version >= 7 ? 16 : 1);

        stream.Position = blocksInfoOffset;
        return ReadExactly(stream, checked((int)header.CompressedBlocksInfoSize));
    }

    private static BlocksInfo ParseBlocksInfo(byte[] bytes)
    {
        using var stream = new MemoryStream(bytes, writable: false);
        var uncompressedDataHash = ReadExactly(stream, 16);

        var blockCount = ReadInt32BigEndian(stream);
        var blocks = new List<StorageBlockInfo>(blockCount);
        for (var index = 0; index < blockCount; index++)
        {
            blocks.Add(new StorageBlockInfo(ReadUInt32BigEndian(stream),
                                            ReadUInt32BigEndian(stream),
                                            ReadUInt16BigEndian(stream)));
        }

        var nodeCount = ReadInt32BigEndian(stream);
        var nodes = new List<NodeInfo>(nodeCount);
        for (var index = 0; index < nodeCount; index++)
        {
            nodes.Add(new NodeInfo(ReadInt64BigEndian(stream), ReadInt64BigEndian(stream),
                                   ReadUInt32BigEndian(stream),
                                   ReadNullTerminatedString(stream)));
        }

        return new BlocksInfo(uncompressedDataHash, blocks, nodes);
    }

    private static byte[] BuildBlocksInfoBytes(BlocksInfo blocksInfo)
    {
        using var stream = new MemoryStream();
        stream.Write(blocksInfo.UncompressedDataHash);
        WriteInt32BigEndian(stream, blocksInfo.Blocks.Count);
        foreach (var block in blocksInfo.Blocks)
        {
            WriteUInt32BigEndian(stream, block.UncompressedSize);
            WriteUInt32BigEndian(stream, block.CompressedSize);
            WriteUInt16BigEndian(stream, block.Flags);
        }

        WriteInt32BigEndian(stream, blocksInfo.Nodes.Count);
        foreach (var node in blocksInfo.Nodes)
        {
            WriteInt64BigEndian(stream, node.Offset);
            WriteInt64BigEndian(stream, node.Size);
            WriteUInt32BigEndian(stream, node.Flags);
            WriteNullTerminatedString(stream, node.Path);
        }

        return stream.ToArray();
    }

    private static byte[][] DecryptBlocksInParallel(string inputPath, IReadOnlyList<BlockJob> blockJobs,
                                                    byte[] keyBytes, BlocksInfo blocksInfo)
    {
        var decryptedBlocks = new byte[blockJobs.Count][];
        var parallelism = Math.Min(Environment.ProcessorCount, Math.Max(1, blockJobs.Count));

        Parallel.ForEach(
            Enumerable.Range(0, blockJobs.Count),
            new ParallelOptions { MaxDegreeOfParallelism = parallelism },
            () => File.OpenRead(inputPath),
            (index, _, localInput) =>
            {
                var job = blockJobs[index];
                localInput.Position = job.Offset;
                var encryptedBytes = ReadExactly(localInput, checked((int)job.EncryptedSize));
                var decryptedBytes = DecryptBlockBytes(encryptedBytes, keyBytes);
                blocksInfo.Blocks[index].CompressedSize = (uint)decryptedBytes.Length;
                decryptedBlocks[index] = decryptedBytes;
                return localInput;
            },
            localInput => localInput.Dispose());

        return decryptedBlocks;
    }

    private static byte[] DecryptBlockBytes(byte[] encryptedBytes, byte[] keyBytes)
    {
        using var aes = new AesGcm(keyBytes, 16);
        using var output = new MemoryStream();

        var cursor = 0;
        while (cursor < encryptedBytes.Length)
        {
            var remaining = encryptedBytes.Length - cursor;
            var chunkSize = Math.Min(remaining, EncryptedBlockUnitSize);
            var chunk = encryptedBytes.AsSpan(cursor, chunkSize);

            if (chunk.Length < 32)
            {
                throw new InvalidOperationException(
                    "Encountered an encrypted chunk that is too small.");
            }

            var iv = chunk[..12];
            var plainLength = BinaryPrimitives.ReadInt32LittleEndian(chunk.Slice(12, 4));
            var consumed = 32 + plainLength;
            if (plainLength < 0 || consumed > chunk.Length)
            {
                throw new InvalidOperationException(
                    "Encountered an invalid encrypted chunk layout.");
            }

            var ciphertext = chunk.Slice(16, plainLength);
            var tag = chunk.Slice(16 + plainLength, 16);
            var plain = new byte[plainLength];
            aes.Decrypt(iv, ciphertext, tag, plain);
            output.Write(plain);
            cursor += consumed;
        }

        return output.ToArray();
    }

    private static IReadOnlyList<BlockJob> BuildBlockJobs(IReadOnlyList<StorageBlockInfo> blocks,
                                                          long blockDataStart, long blockDataEnd)
    {
        var jobs = new List<BlockJob>(blocks.Count);
        var offset = blockDataStart;
        foreach (var block in blocks)
        {
            if (offset + block.CompressedSize > blockDataEnd)
            {
                throw new InvalidOperationException("Encountered a truncated encrypted block.");
            }

            jobs.Add(new BlockJob(offset, block.CompressedSize));
            offset += block.CompressedSize;
        }

        return jobs;
    }

    private static long GetBlockDataStartOffset(long fileLength, BundleHeader header)
    {
        var offset = AlignValue(GetHeaderLength(header), header.Version >= 7 ? 16 : 1);
        if ((header.Flags & ArchiveFlags.BlocksInfoAtTheEnd) == 0)
        {
            offset += header.CompressedBlocksInfoSize;
            if ((header.Flags & ArchiveFlags.BlockInfoNeedPaddingAtStart) != 0)
            {
                offset = AlignValue(offset, 16);
            }
        }

        return offset;
    }

    private static long GetBlockDataEndOffset(long fileLength, BundleHeader header)
    {
        return (header.Flags & ArchiveFlags.BlocksInfoAtTheEnd) != 0
                   ? fileLength - header.CompressedBlocksInfoSize
                   : fileLength;
    }

    private static long GetHeaderLength(BundleHeader header)
    {
        return Encoding.UTF8.GetByteCount(header.Signature) + 1 + sizeof(uint) +
               Encoding.UTF8.GetByteCount(header.UnityVersion) + 1 +
               Encoding.UTF8.GetByteCount(header.UnityRevision) + 1 + sizeof(long) +
               sizeof(uint) + sizeof(uint) + sizeof(uint);
    }

    private static long CalculateOutputLength(BundleHeader header, int blocksInfoSize,
                                              IEnumerable<byte[]> decryptedBlocks)
    {
        var total = AlignValue(GetHeaderLength(header), header.Version >= 7 ? 16 : 1);
        if ((header.Flags & ArchiveFlags.BlocksInfoAtTheEnd) == 0)
        {
            total += blocksInfoSize;
            if ((header.Flags & ArchiveFlags.BlockInfoNeedPaddingAtStart) != 0)
            {
                total = AlignValue(total, 16);
            }
        }

        total += decryptedBlocks.Sum(block => (long)block.Length);
        if ((header.Flags & ArchiveFlags.BlocksInfoAtTheEnd) != 0)
        {
            total += blocksInfoSize;
        }

        return total;
    }

    private static string ReadNullTerminatedString(Stream stream, int? maxBytes = null)
    {
        using var buffer = new MemoryStream();
        var remaining = maxBytes ?? int.MaxValue;
        while (remaining-- > 0)
        {
            var next = stream.ReadByte();
            if (next < 0)
            {
                throw new EndOfStreamException(
                    "Unexpected end of stream while reading a string.");
            }

            if (next == 0)
            {
                return Encoding.UTF8.GetString(buffer.ToArray());
            }

            buffer.WriteByte((byte)next);
        }

        throw new InvalidOperationException("Encountered a string without a null terminator.");
    }

    private static void WriteNullTerminatedString(Stream stream, string value)
    {
        var bytes = Encoding.UTF8.GetBytes(value);
        stream.Write(bytes);
        stream.WriteByte(0);
    }

    private static byte[] ReadExactly(Stream stream, int count)
    {
        var bytes = new byte[count];
        var offset = 0;
        while (offset < count)
        {
            var read = stream.Read(bytes, offset, count - offset);
            if (read == 0)
            {
                throw new EndOfStreamException("Unexpected end of stream.");
            }

            offset += read;
        }

        return bytes;
    }

    private static ushort ReadUInt16BigEndian(Stream stream)
    {
        var bytes = ReadExactly(stream, sizeof(ushort));
        return BinaryPrimitives.ReadUInt16BigEndian(bytes);
    }

    private static uint ReadUInt32BigEndian(Stream stream)
    {
        var bytes = ReadExactly(stream, sizeof(uint));
        return BinaryPrimitives.ReadUInt32BigEndian(bytes);
    }

    private static int ReadInt32BigEndian(Stream stream)
    {
        var bytes = ReadExactly(stream, sizeof(int));
        return BinaryPrimitives.ReadInt32BigEndian(bytes);
    }

    private static long ReadInt64BigEndian(Stream stream)
    {
        var bytes = ReadExactly(stream, sizeof(long));
        return BinaryPrimitives.ReadInt64BigEndian(bytes);
    }

    private static void WriteUInt16BigEndian(Stream stream, ushort value)
    {
        Span<byte> bytes = stackalloc byte[sizeof(ushort)];
        BinaryPrimitives.WriteUInt16BigEndian(bytes, value);
        stream.Write(bytes);
    }

    private static void WriteUInt32BigEndian(Stream stream, uint value)
    {
        Span<byte> bytes = stackalloc byte[sizeof(uint)];
        BinaryPrimitives.WriteUInt32BigEndian(bytes, value);
        stream.Write(bytes);
    }

    private static void WriteInt32BigEndian(Stream stream, int value)
    {
        Span<byte> bytes = stackalloc byte[sizeof(int)];
        BinaryPrimitives.WriteInt32BigEndian(bytes, value);
        stream.Write(bytes);
    }

    private static void WriteInt64BigEndian(Stream stream, long value)
    {
        Span<byte> bytes = stackalloc byte[sizeof(long)];
        BinaryPrimitives.WriteInt64BigEndian(bytes, value);
        stream.Write(bytes);
    }

    private static void AlignStream(Stream stream, int alignment)
    {
        var aligned = AlignValue(stream.Position, alignment);
        while (stream.Position < aligned)
        {
            stream.WriteByte(0);
        }
    }

    private static long AlignValue(long value, int alignment)
    {
        if (alignment <= 1)
        {
            return value;
        }

        var remainder = value % alignment;
        return remainder == 0 ? value : value + alignment - remainder;
    }

    private static string ResolveDecryptOutputPath(string inputPath,
                                                   string? configuredOutputPath)
    {
        if (!string.IsNullOrWhiteSpace(configuredOutputPath))
        {
            return configuredOutputPath;
        }

        return BuildDefaultDecryptedFilePath(inputPath);
    }

    private static string BuildDefaultDecryptedFilePath(string inputPath)
    {
        var directory = Path.GetDirectoryName(inputPath) ?? Directory.GetCurrentDirectory();
        var fileName = Path.GetFileNameWithoutExtension(inputPath);
        var extension = Path.GetExtension(inputPath);
        return Path.Combine(directory, $"{fileName}.decrypted{extension}");
    }

    private sealed record DecryptResult(string InputPath, string OutputPath);
    private sealed record BundleHeader(string Signature, uint Version, string UnityVersion,
                                       string UnityRevision, long Size,
                                       uint CompressedBlocksInfoSize,
                                       uint UncompressedBlocksInfoSize, ArchiveFlags Flags);
    private sealed record StorageBlockInfo(uint UncompressedSize, uint CompressedSize,
                                           ushort Flags)
    {
        public uint CompressedSize { get; set; } = CompressedSize;
    }
    private sealed record NodeInfo(long Offset, long Size, uint Flags, string Path);
    private sealed record BlocksInfo(byte[] UncompressedDataHash, List<StorageBlockInfo> Blocks,
                                     List<NodeInfo> Nodes);
    private sealed record BlockJob(long Offset, uint EncryptedSize);
}
