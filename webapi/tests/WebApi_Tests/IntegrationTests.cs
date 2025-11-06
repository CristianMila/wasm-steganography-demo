using Microsoft.AspNetCore.Mvc.Testing;
using System.Net.Http.Json;

namespace WebApi_Tests;

public class IntegrationTests
{
    [Fact]
    public async Task EncodeSecretIntoBmp_ValidBmp_ReturnsEncodedByteArray()
    {
        var host = new WebApplicationFactory<Program>().CreateDefaultClient();
        var img = File.ReadAllBytes("Data/test.bmp");
        var serializedBmp = Convert.ToBase64String(img);
        var req = new EncodeReq("foo", serializedBmp, "image/bmp");
        var response = await host.PostAsync("/encode", JsonContent.Create(req), TestContext.Current.CancellationToken);

        if (!response.IsSuccessStatusCode)
        {
            var errorContent = await response.Content.ReadAsStringAsync(TestContext.Current.CancellationToken);
            var errorMessage = $"Request failed with status code {response.StatusCode}. " +
            $"Response: {errorContent}";
            Assert.Fail(errorMessage);
        }

        response.EnsureSuccessStatusCode();

        var alreadyEncodedBmp = File.ReadAllBytes("Data/test-encoded.bmp");
        var answeredEncodedBmp = await response.Content.ReadAsByteArrayAsync(TestContext.Current.CancellationToken);

        Assert.Equivalent(alreadyEncodedBmp, answeredEncodedBmp);
    }

    [Fact]
    public async Task DecodeSecretFromBmp_EncodedBmp_ReturnsSecret()
    {
        var host = new WebApplicationFactory<Program>().CreateDefaultClient();
        var img = File.ReadAllBytes("Data/test-encoded.bmp");
        var req = new ByteArrayContent(img);
        req.Headers.ContentType = new System.Net.Http.Headers.MediaTypeHeaderValue("image/bmp");
        var response = await host.PostAsync("/decode", req, TestContext.Current.CancellationToken);

        if (!response.IsSuccessStatusCode)
        {
            var errorContent = await response.Content.ReadAsStringAsync(TestContext.Current.CancellationToken);
            var errorMessage = $"Request failed with status code {response.StatusCode}. " +
            $"Response: {errorContent}";
            Assert.Fail(errorMessage);
        }

        var answeredSecret = await response.Content.ReadAsStringAsync(TestContext.Current.CancellationToken);

        Assert.Equal("foo", answeredSecret.Trim('\"'));
    }

    [Fact]
    public async Task DecodeSecretFromBmp_UnencodedBmp_Fails()
    {
        var host = new WebApplicationFactory<Program>().CreateDefaultClient();
        var img = File.ReadAllBytes("Data/test.bmp");
        var req = new ByteArrayContent(img);
        req.Headers.ContentType = new System.Net.Http.Headers.MediaTypeHeaderValue("image/bmp");
        var response = await host.PostAsync("/decode", req, TestContext.Current.CancellationToken);

        // wasm function should fail better... this means nothing.
        Assert.Equal(System.Net.HttpStatusCode.InternalServerError, response.StatusCode);
    }

    [Fact]
    public async Task EncodeSecretIntoJpeg_ValidJpeg_ReturnsEncodedByteArray()
    {
        var host = new WebApplicationFactory<Program>().CreateDefaultClient();
        var img = File.ReadAllBytes("Data/test.jpg");
        var serializedJpeg = Convert.ToBase64String(img);
        var req = new EncodeReq("foo", serializedJpeg, "image/jpeg");
        var response = await host.PostAsync("/encode", JsonContent.Create(req), TestContext.Current.CancellationToken);

        if (!response.IsSuccessStatusCode)
        {
            var errorContent = await response.Content.ReadAsStringAsync(TestContext.Current.CancellationToken);
            var errorMessage = $"Request failed with status code {response.StatusCode}. " +
            $"Response: {errorContent}";
            Assert.Fail(errorMessage);
        }

        response.EnsureSuccessStatusCode();

        var alreadyEncodedJpeg = File.ReadAllBytes("Data/test-encoded.jpg");
        var answeredEncodedJpeg = await response.Content.ReadAsByteArrayAsync(TestContext.Current.CancellationToken);

        Assert.Equivalent(alreadyEncodedJpeg, answeredEncodedJpeg);
    }

    [Fact]
    public async Task DecodeSecretFromJpeg_EncodedJpeg_ReturnsSecret()
    {
        var host = new WebApplicationFactory<Program>().CreateDefaultClient();
        var img = File.ReadAllBytes("Data/test-encoded.jpg");
        var req = new ByteArrayContent(img);
        req.Headers.ContentType = new System.Net.Http.Headers.MediaTypeHeaderValue("image/jpg");
        var response = await host.PostAsync("/decode", req, TestContext.Current.CancellationToken);

        if (!response.IsSuccessStatusCode)
        {
            var errorContent = await response.Content.ReadAsStringAsync(TestContext.Current.CancellationToken);
            var errorMessage = $"Request failed with status code {response.StatusCode}. " +
            $"Response: {errorContent}";
            Assert.Fail(errorMessage);
        }

        var answeredSecret = await response.Content.ReadAsStringAsync(TestContext.Current.CancellationToken);

        Assert.Equal("foo", answeredSecret.Trim('\"'));
    }

    [Fact]
    public async Task DecodeSecretFromJpeg_UnencodedJpeg_Fails()
    {
        var host = new WebApplicationFactory<Program>().CreateDefaultClient();
        var img = File.ReadAllBytes("Data/test.jpg");
        var req = new ByteArrayContent(img);
        req.Headers.ContentType = new System.Net.Http.Headers.MediaTypeHeaderValue("image/jpeg");
        var response = await host.PostAsync("/decode", req, TestContext.Current.CancellationToken);

        // wasm function should fail better... this means nothing.
        Assert.Equal(System.Net.HttpStatusCode.InternalServerError, response.StatusCode);
    }
}

