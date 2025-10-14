using System.Buffers.Text;
using Microsoft.AspNetCore.Mvc;

const string ENV_WASM_MODULE_PATH = "WASM_STEGANOGRAPHY_FILE_PATH";

var builder = WebApplication.CreateBuilder(args);

builder.Configuration.AddEnvironmentVariables();
builder.Services.AddOpenApi();
builder.Services.AddSingleton<SteganographyWasmModule>((serviceProvider) =>
{
    var env = serviceProvider.GetRequiredService<IConfiguration>();
    var webEnv = serviceProvider.GetRequiredService<IWebHostEnvironment>();
    var wasmPath = env.GetValue<string>(ENV_WASM_MODULE_PATH);

    if (!File.Exists(wasmPath))
    {
        throw new FileNotFoundException("Failed locating the WASM modules.");
    }

    return new SteganographyWasmModule(wasmPath);
});
builder.Services.AddMvc();

var app = builder.Build();

if (app.Environment.IsDevelopment())
{
    app.MapOpenApi();
}

app.UseHttpsRedirection();

app.MapPost("/encode", async ([FromBody] EncodeReq req, SteganographyWasmModule steganographyModule) =>
{
    var span = System.Text.Encoding.UTF8.GetBytes(req.Bmp);
    var image = Base64.DecodeFromUtf8InPlace(span, out int bytesWritten);
    var encodedImage = await steganographyModule.Encode(req.Secret, span.ToArray());

    return Results.File(encodedImage, "image/bmp", "encoded_image.bmp");
});

app.MapPost("/decode", async (HttpRequest request, SteganographyWasmModule steganographyModule) =>
{
    if (!request.HasFormContentType && request.ContentType != "image/bmp")
    {
        return Results.BadRequest("Expected 'image/bmp' content type.");
    }

    using var memoryStream = new MemoryStream();
    await request.Body.CopyToAsync(memoryStream);
    var imageBytes = memoryStream.ToArray();
    var decodedSecret = await steganographyModule.Decode(imageBytes);

    return Results.Ok(decodedSecret);
}).Accepts<byte[]>("image/bmp");

app.Run();

public record EncodeReq(string Secret, string Bmp);
