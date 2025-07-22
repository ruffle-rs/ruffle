package {
import flash.display.*;
import flash.geom.*;

public class Test extends MovieClip {
    [Embed(source = "conditional_if.pbj", mimeType="application/octet-stream")]
    public static var ShaderIf: Class;
    [Embed(source = "conditional_select.pbj", mimeType="application/octet-stream")]
    public static var ShaderSelect: Class;

    [Embed(source = "conditional_if_float.pbj", mimeType="application/octet-stream")]
    public static var ShaderIfFloat: Class;
    [Embed(source = "conditional_select_float.pbj", mimeType="application/octet-stream")]
    public static var ShaderSelectFloat: Class;

    [Embed(source = "conditional_if_wide.pbj", mimeType="application/octet-stream")]
    public static var ShaderIfWide: Class;
    [Embed(source = "conditional_select_wide.pbj", mimeType="application/octet-stream")]
    public static var ShaderSelectWide: Class;

    public function Test() {
        testValue(0.0);
        testValue(0.1);
        testValue(0.5);
        testValue(0.9);
        testValue(1.0);
        testValue(1.1);
        testValue(1.5);
        testValue(1.9);
        testValue(2.0);
        testValue(2.5);
        testValue(-0.1);
        testValue(-0.5);
        testValue(-0.9);
        testValue(-1.0);
        testValue(-1.1);
        testValue(-1.5);
        testValue(-2.0);
        testValue(-2.5);

        testValueInWideShaders([0, 0, 0, 0]);
        testValueInWideShaders([1, 0, 0, 0]);
        testValueInWideShaders([0, 1, 0, 0]);
        testValueInWideShaders([0, 0, 1, 0]);
        testValueInWideShaders([0, 0, 0, 1]);
        testValueInWideShaders([2, 0, 0, 0]);
        testValueInWideShaders([0, 2, 0, 0]);
        testValueInWideShaders([0, 0, 2, 0]);
        testValueInWideShaders([0, 0, 0, 2]);
        testValueInWideShaders([2, 1, 0, 0]);
        testValueInWideShaders([0, 2, 1, 0]);
        testValueInWideShaders([0, 0, 2, 1]);
        testValueInWideShaders([1, 0, 0, 2]);

        try {
            new Shader(new ShaderIfFloat());
            trace("ShaderIfFloat compiled");
        } catch (e) {
            trace("Error compiling ShaderIfFloat");
            trace(e.getStackTrace());
        }

        try {
            new Shader(new ShaderSelectFloat());
            trace("ShaderSelectFloat compiled");
        } catch (e) {
            trace("Error compiling ShaderSelectFloat");
            trace(e.getStackTrace());
        }
    }

    private function testValue(value:Number) {
        testValueInShader(value, new Shader(new ShaderIf()), "if");
        testValueInShader(value, new Shader(new ShaderSelect()), "select");
    }

    private function testValueInShader(value:Number, shader:Shader, name:String) {
        var input = new BitmapData(1, 1);
        var shaderJob = new ShaderJob(shader, input);

        shaderJob.shader.data.boolInput.value = [value];
        shaderJob.shader.data.intInput.value = [value];

        try {
            shaderJob.start(true);
            trace(name + ": " + value + " -> " + input.getPixel32(0, 0).toString(16));
        } catch (e) {
            trace("Error while starting: " + e);
        }
        trace("=================");
    }

    private function testValueInWideShaders(values:Array) {
        testValueInWideShader(values, new Shader(new ShaderIfWide()), "if_wide");
        testValueInWideShader(values, new Shader(new ShaderSelectWide()), "select_wide");
    }

    private function testValueInWideShader(values:Array, shader:Shader, name:String) {
        var input = new BitmapData(1, 1);
        var shaderJob = new ShaderJob(shader, input);

        shaderJob.shader.data.int4Input.value = values;

        try {
            shaderJob.start(true);
            trace(name + ": " + values + " -> " + input.getPixel32(0, 0).toString(16));
        } catch (e) {
            trace("Error while starting: " + e);
        }
        trace("=================");
    }
}
}
