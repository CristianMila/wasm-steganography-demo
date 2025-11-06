using Wasmtime;
using System.Text;

public class SteganographyWasmModule : IDisposable
{
    private readonly Store _store;
    private readonly Instance _instance;
    private readonly Memory _memory;
    private readonly Func<int, int, int, int, int> _encodeSecretIntoBmp;
    private readonly Func<int, int, int, int, int> _encodeSecretIntoJpeg;
    private readonly Func<int, int, int> _decodeSecretFromBmp;
    private readonly Func<int, int, int> _decodeSecretFromJpeg;
    private readonly Func<int, int, int, int, int> _cabi_realloc;
    private readonly SemaphoreSlim _asyncLock = new SemaphoreSlim(1, 1);

    public SteganographyWasmModule(string modulePath)
    {
        var engine = new Engine();
        _store = new Store(engine);

        var wasiConf = new WasiConfiguration();
        _store.SetWasiConfiguration(wasiConf);

        var linker = new Linker(engine);
        var memory = new Memory(_store, 1);
        linker.DefineWasi();
        linker.Define("env", "memory", memory);

        var module = Module.FromFile(engine, modulePath);
        linker.DefineFunction("local:steganography", "log", (string? message) => Console.WriteLine($"[WASM]: {message}"));
        linker.DefineFunction("$root", "log", (int ptr, int len) =>
        {
            // var message = memory.ReadString(ptr, len);
            throw new Exception($"log call: ptr={ptr}, len={len}");
        });

        _instance = linker.Instantiate(_store, module);
        _memory = _instance.GetMemory("memory")
        ?? throw new InvalidOperationException("No se encontró la memoria exportada.");
        _encodeSecretIntoBmp = _instance.GetFunction<int, int, int, int, int>("encode-secret-into-bmp")
        ?? throw new InvalidOperationException("No se encontró la función de codificación.");
        _decodeSecretFromBmp = _instance.GetFunction<int, int, int>("decode-secret-from-bmp")
        ?? throw new InvalidOperationException("No se encontró la función de decodificación.");
        _encodeSecretIntoJpeg = _instance.GetFunction<int, int, int, int, int>("encode-secret-into-jpeg")
        ?? throw new InvalidOperationException("No se encontró la función de codificación.");
        _decodeSecretFromJpeg = _instance.GetFunction<int, int, int>("decode-secret-from-jpeg")
        ?? throw new InvalidOperationException("No se encontró la función de decodificación.");
        _cabi_realloc = _instance.GetFunction<int, int, int, int, int>("cabi_realloc")
            ?? throw new InvalidOperationException("No se encontró la función de alloc.");
    }

    public async Task<string> DecodeFromBmp(byte[] image)
    {
        await _asyncLock.WaitAsync();

        try
        {
            // write parameters into the module's memory
            var (imgPtr, imgLen) = WriteByteArray(image);

            // call the module's exported function
            int resultPtr = _decodeSecretFromBmp((int)imgPtr, (int)imgLen);

            // read the result, encoded as a 32bit integer:
            // - 2 high bytes represent the memory pointer to the result
            int resultAddress = _memory.ReadInt32(resultPtr);

            // - 2 low bytes represent the length of the data
            int resultLength = _memory.ReadInt32(resultPtr + 4);

            // get the decoded string encoded in the byte array pointed by the result
            var secret = _memory.ReadString(resultAddress, resultLength);

            return secret;
        }
        finally
        {
            _asyncLock.Release();
        }
    }

    public async Task<byte[]> EncodeIntoBmp(string secret, byte[] image)
    {
        await _asyncLock.WaitAsync();

        try
        {
            // write parameters into the module's memory
            var (secretPtr, secretLen) = WriteString(secret);
            var (imgPtr, imgLen) = WriteByteArray(image);

            // call the module's exported function
            var resultPtr = _encodeSecretIntoBmp((int)secretPtr, (int)secretLen, (int)imgPtr, (int)imgLen);

            // read the result, encoded as a 32bit integer:
            // - 2 high bytes represent the memory pointer to the result
            var resultAddress = _memory.ReadInt32(resultPtr);

            // - 2 low bytes represent the length of the data
            var resultLength = _memory.ReadInt32(resultPtr + 4);

            // get the resulting byte array
            var outputImage = _memory.GetSpan(resultAddress, resultLength).ToArray();

            return outputImage;
        }
        finally
        {
            _asyncLock.Release();
        }
    }

    public async Task<string> DecodeFromJpeg(byte[] image)
    {
        await _asyncLock.WaitAsync();

        try
        {
            // write parameters into the module's memory
            var (imgPtr, imgLen) = WriteByteArray(image);

            // call the module's exported function
            int resultPtr = _decodeSecretFromJpeg((int)imgPtr, (int)imgLen);

            // read the result, encoded as a 32bit integer:
            // - 2 high bytes represent the memory pointer to the result
            int resultAddress = _memory.ReadInt32(resultPtr);

            // - 2 low bytes represent the length of the data
            int resultLength = _memory.ReadInt32(resultPtr + 4);

            // get the decoded string encoded in the byte array pointed by the result
            var secret = _memory.ReadString(resultAddress, resultLength);

            return secret;
        }
        finally
        {
            _asyncLock.Release();
        }
    }

    public async Task<byte[]> EncodeIntoJpeg(string secret, byte[] image)
    {
        await _asyncLock.WaitAsync();

        try
        {
            // write parameters into the module's memory
            var (secretPtr, secretLen) = WriteString(secret);
            var (imgPtr, imgLen) = WriteByteArray(image);

            // call the module's exported function
            var resultPtr = _encodeSecretIntoJpeg((int)secretPtr, (int)secretLen, (int)imgPtr, (int)imgLen);

            // read the result, encoded as a 32bit integer:
            // - 2 high bytes represent the memory pointer to the result
            var resultAddress = _memory.ReadInt32(resultPtr);

            // - 2 low bytes represent the length of the data
            var resultLength = _memory.ReadInt32(resultPtr + 4);

            // get the resulting byte array
            var outputImage = _memory.GetSpan(resultAddress, resultLength).ToArray();

            return outputImage;
        }
        finally
        {
            _asyncLock.Release();
        }
    }

    private int AllocateBytes(uint size)
    {
        long currentLength = _memory.GetLength();
        long address = currentLength;

        // grow the memory if needed
        if (currentLength + size > _memory.GetSize() * Memory.PageSize)
        {

            long pagesNeeded = ((currentLength + size) - _memory.GetSize() * Memory.PageSize + Memory.PageSize - 1) / Memory.PageSize;
            _memory.Grow(pagesNeeded);
        }

        return _cabi_realloc(0, 0, 1, (int)size);
    }

    private (nint, uint) WriteString(string s)
    {
        var sEncoding = Encoding.UTF8.GetBytes(s);
        var ptr = AllocateBytes((uint)sEncoding.Length);
        var bytesWritten = _memory.WriteString(ptr, s);

        return ((nint)ptr, (uint)bytesWritten);
    }

    private (nint, uint) WriteByteArray(byte[] array)
    {
        var ptr = AllocateBytes((uint)array.Length);
        ptr = _cabi_realloc(0, 0, 1, array.Length);
        array.CopyTo(_memory.GetSpan(ptr, (int)array.Length));

        return ((nint)ptr, (uint)array.Length);
    }

    private byte[] ReadByteArray(nint ptr, uint len) => _memory.GetSpan(ptr, (int)len).ToArray();

    public void Dispose() => _store.Dispose();
}
