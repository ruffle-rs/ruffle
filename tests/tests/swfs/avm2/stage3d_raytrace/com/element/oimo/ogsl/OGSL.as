/* Copyright (c) 2015 EL-EMENT saharan
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of this
 * software and associated documentation  * files (the "Software"), to deal in the Software
 * without restriction, including without limitation the rights to use, copy,  * modify, merge,
 * publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to
 * whom the Software is furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all copies or
 * substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED,
 * INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR
 * PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR
 * ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,
 * ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */
package com.element.oimo.ogsl {
	import flash.display3D.*;
	import flash.display3D.textures.*;
	import flash.geom.*;
	import flash.utils.*;

	/**
	 * OGSL - Oimo Graphics Shader Language:
	 *   a high-level shading language for Stage3D shader programming
	 * @author saharan
	 */
	public class OGSL {
		private var ogslCode:String;
		private var tokenizer:OGSLTokenizer;
		private var parser:OGSLParser;
		private var analyzer:OGSLAnalyzer;
		private var environment:OGSLEnvironment;
		private var emitter:OGSLEmitter;
		private var optimizer:OGSLOptimizer;
		private var vertexOutput:OGSLOutput;
		private var fragmentOutput:OGSLOutput;
		private var c3d:Context3D;
		private var registerMap:OGSLRegisterMap;
		private var logFunction:Function;
		private var compiled:Boolean;
		private const tmpVector:Vector.<Number> = new Vector.<Number>(4, true);

		public function OGSL() {
			tokenizer = new OGSLTokenizer();
			parser = new OGSLParser();
			analyzer = new OGSLAnalyzer();
			emitter = new OGSLEmitter();
			optimizer = new OGSLOptimizer(log);
			registerMap = new OGSLRegisterMap();
		}

		/**
		 * set log function used for logging
		 * @param	logFunction function (logText:String):void { ... }
		 */
		public function setLogFunction(logFunction:Function):void {
			this.logFunction = logFunction;
		}

		/**
		 * compile given OGSL source code
		 * @param	ogslCode OGSL source code
		 * @see	isCompiled true if the compiling is finished without any error
		 */
		public function compile(ogslCode:String):void {
			compiled = false;
			vertexOutput = null;
			fragmentOutput = null;
			this.ogslCode = ogslCode.replace(/\r|\r\n/g, "\n");
			try {
				log("");
				log("input code:");
				log(ogslCode);
				log("");
				tokenize();
				parse();
				analyze();
				emit();
				//optimize();
				map();
				compiled = true;
			} catch (e:Error) {
				throw e;
			}
		}

		/**
		 * return if the last compiling is finished without any error
		 * @return	true if no error
		 */
		public function isCompiled():Boolean {
			return compiled;
		}

		/**
		 * return the last-compiled AGAL of vertex shader
		 * @return	vertex shader AGAL
		 */
		public function getVertexAGAL():String {
			if (!vertexOutput) throw new Error("compiling is not finished");
			return vertexOutput.print();
		}

		/**
		 * return the last-compiled AGAL of fragment shader
		 * @return	fragment shader AGAL
		 */
		public function getFragmentAGAL():String {
			if (!fragmentOutput) throw new Error("compiling is not finished");
			return fragmentOutput.print();
		}

		/**
		 * set Context3D object
		 * @param	c3d Context3D
		 */
		public function setContext3D(c3d:Context3D):void {
			this.c3d = c3d;
		}

		/**
		 * set vertex buffer to Context3D
		 * @param	name vertex attribute name in OGSL source
		 * @param	buffer VertexBuffer3D
		 * @param	bufferOffset buffer start index
		 * @param	format Context3DVertexBufferFormat
		 * @see	setContext3D
		 */
		public function setVertexBuffer(name:String, buffer:VertexBuffer3D, bufferOffset:int, format:String):void {
			if (!c3d) throw new Error("Context3D is not set");
			c3d.setVertexBufferAt(getVertexBufferIndex(name), buffer, bufferOffset, format);
		}

		/**
		 * set vertex constants to Context3D from Number
		 * @param	name vertex uniform name in OGSL source
		 * @param	data data to set
		 * @see	setContext3D
		 */
		public function setVertexConstantsFromNumber(name:String, data:Number):void {
			if (!c3d) throw new Error("Context3D is not set");
			tmpVector[0] = data;
			tmpVector[1] = 0;
			tmpVector[2] = 0;
			tmpVector[3] = 0;
			c3d.setProgramConstantsFromVector(Context3DProgramType.VERTEX, getVertexConstantsIndex(name), tmpVector);
		}

		/**
		 * set vertex constants to Context3D from Vector.<Number>
		 * @param	name vertex uniform name in OGSL source
		 * @param	data data to set
		 * @see	setContext3D
		 */
		public function setVertexConstantsFromVector(name:String, data:Vector.<Number>):void {
			if (!c3d) throw new Error("Context3D is not set");
			var num:int = data.length;
			if (num < 4) {
				for (var i:int = 0; i < num; i++) {
					tmpVector[i] = data[i];
				}
				while (i < 4) {
					tmpVector[i] = 0;
					i++;
				}
				data = tmpVector;
			}
			c3d.setProgramConstantsFromVector(Context3DProgramType.VERTEX, getVertexConstantsIndex(name), data);
		}

		/**
		 * set vertex constants to Context3D from Matrix3D
		 * @param	name vertex uniform name in OGSL source
		 * @param	matrix matrix to set
		 * @param	transposedMatrix true if the matrix is transposed
		 * @see	setContext3D
		 */
		public function setVertexConstantsFromMatrix(name:String, matrix:Matrix3D, transposedMatrix:Boolean = false):void {
			if (!c3d) throw new Error("Context3D is not set");
			c3d.setProgramConstantsFromMatrix(Context3DProgramType.VERTEX, getVertexConstantsIndex(name), matrix, transposedMatrix);
		}

		/**
		 * set fragment constants to Context3D from Number
		 * @param	name fragment uniform name in OGSL source
		 * @param	data data to set
		 * @see	setContext3D
		 */
		public function setFragmentConstantsFromNumber(name:String, data:Number):void {
			if (!c3d) throw new Error("Context3D is not set");
			tmpVector[0] = data;
			tmpVector[1] = 0;
			tmpVector[2] = 0;
			tmpVector[3] = 0;
			c3d.setProgramConstantsFromVector(Context3DProgramType.FRAGMENT, getFragmentConstantsIndex(name), tmpVector);
		}

		/**
		 * set fragment constants to Context3D from Vector.<Number>
		 * @param	name fragment uniform name in OGSL source
		 * @param	data data to set
		 * @see	setContext3D
		 */
		public function setFragmentConstantsFromVector(name:String, data:Vector.<Number>):void {
			if (!c3d) throw new Error("Context3D is not set");
			var num:int = data.length;
			if (num < 4) {
				for (var i:int = 0; i < num; i++) {
					tmpVector[i] = data[i];
				}
				while (i < 4) {
					tmpVector[i] = 0;
					i++;
				}
				data = tmpVector;
			}
			c3d.setProgramConstantsFromVector(Context3DProgramType.FRAGMENT, getFragmentConstantsIndex(name), data);
		}

		/**
		 * set fragment constants to Context3D from Matrix3D
		 * @param	name fragment uniform name in OGSL source
		 * @param	matrix matrix to set
		 * @param	transposedMatrix true if the matrix is transposed
		 * @see	setContext3D
		 */
		public function setFragmentConstantsFromMatrix(name:String, matrix:Matrix3D, transposedMatrix:Boolean = false):void {
			if (!c3d) throw new Error("Context3D is not set");
			c3d.setProgramConstantsFromMatrix(Context3DProgramType.FRAGMENT, getFragmentConstantsIndex(name), matrix, transposedMatrix);
		}

		/**
		 * set default (hard-coded) fragment and vertex constants to Context3D
		 * @see	setContext3D
		 */
		public function setDefaultConstants():void {
			if (!c3d) throw new Error("Context3D is not set");
			var data:Vector.<Number>;
			// set vertex constants
			data = getDefaultVertexConstantsData();
			for (var i:int = 0; i < data.length; i += 5) {
				c3d.setProgramConstantsFromVector(Context3DProgramType.VERTEX, data[i], new <Number>[data[i + 1], data[i + 2], data[i + 3], data[i + 4]]);
			}
			// set fragment constants
			data = getDefaultFragmentConstantsData();
			for (i = 0; i < data.length; i += 5) {
				c3d.setProgramConstantsFromVector(Context3DProgramType.FRAGMENT, data[i], new <Number>[data[i + 1], data[i + 2], data[i + 3], data[i + 4]]);
			}
		}

		/**
		 * set given TextureBase to Context3D
		 * @param	name fragment uniform name in OGSL source
		 * @param	texture texture to set
		 * @see	setContext3D
		 */
		public function setTexture(name:String, texture:TextureBase):void {
			if (!c3d) throw new Error("Context3D is not set");
			c3d.setTextureAt(getTextureIndex(name), texture);
		}

		/**
		 * return vertex attribute register (va) index associated with the name
		 * @param	name vertex attribute name in OGSL source
		 * @return	index
		 */
		public function getVertexBufferIndex(name:String):int {
			if (!registerMap) throw new Error("compiling is not finished");
			if (registerMap.vertexBufferIndexMap[name] == null) throw new Error("no such vertex attribute: " + name);
			return registerMap.vertexBufferIndexMap[name];
		}

		/**
		 * return vertex constants register (vc) index associated with the name
		 * @param	name vertex uniform name in OGSL source
		 * @return	index
		 */
		public function getVertexConstantsIndex(name:String):int {
			if (!registerMap) throw new Error("compiling is not finished");
			if (registerMap.vertexConstantsIndexMap[name] == null) throw new Error("no such vertex constants: " + name);
			return registerMap.vertexConstantsIndexMap[name];
		}

		/**
		 * return fragment constants register (fc) index associated with the name
		 * @param	name fragment uniform name in OGSL source
		 * @return	index
		 */
		public function getFragmentConstantsIndex(name:String):int {
			if (!registerMap) throw new Error("compiling is not finished");
			if (registerMap.fragmentConstantsIndexMap[name] == null) throw new Error("no such fragment constants: " + name);
			return registerMap.fragmentConstantsIndexMap[name];
		}

		/**
		 * return default (hard-coded) vertex constants register (vc) data
		 * format:
		 *   [
		 *     index1, x1, y1, z1, w1,
		 *     index2, x2, y2, z2, w2,
		 *     ...
		 *   ]
		 * @return index and value data
		 */
		public function getDefaultVertexConstantsData():Vector.<Number> {
			if (!registerMap) throw new Error("compiling is not finished");
			return registerMap.defaultConstantsDataVertex;
		}

		/**
		 * return default (hard-coded) fragment constants register (fc) data
		 * format:
		 *   [
		 *     index1, x1, y1, z1, w1,
		 *     index2, x2, y2, z2, w2,
		 *     ...
		 *   ]
		 * @return index and value data
		 */
		public function getDefaultFragmentConstantsData():Vector.<Number> {
			if (!registerMap) throw new Error("compiling is not finished");
			return registerMap.defaultConstantsDataFragment;
		}

		/**
		 * return fragment sampling register (fs) index associated with the name
		 * @param	name fragment uniform name in OGSL source
		 * @return	index
		 */
		public function getTextureIndex(name:String):int {
			if (!registerMap) throw new Error("compiling is not finished");
			if (registerMap.textureIndexMap[name] == null) throw new Error("no such texture: " + name);
			return registerMap.textureIndexMap[name];
		}

		private function tokenize():void {
			log("");
			log("tokenizing...");
			tokenizer.tokenize(ogslCode);
			log("tokenized.");
			log("");
			log("tokens:");
			var text:String = "";
			var tokens:OGSLToken = tokenizer.tokens;
			while (tokens) {
				text += tokens;
				tokens = tokens.next;
			}
			log(text);
		}

		private function parse():void {
			log("");
			log("parsing...");
			parser.parse(tokenizer.tokens);
			log("parsed.");
			log("");
			log("syntax tree:");
			log(JSON.stringify(parser.nodes));
		}

		private function analyze():void {
			log("");
			log("analyzing...");
			analyzer.analyze(parser.nodes);
			environment = analyzer.env;
			log("analyzed.");
		}

		private function emit():void {
			log("");
			log("emitting...");
			emitter.emit(environment);
			vertexOutput = emitter.vertexOutput;
			fragmentOutput = emitter.fragmentOutput;
			log("emitted.");
			log("");
			log("emitted vertex agal:");
			log(vertexOutput.print());
			log("");
			log("emitted fragment agal:");
			log(fragmentOutput.print());
		}

		private function optimize():void {
			log("");
			log("optimizing...");
			optimizer.optimize(vertexOutput, OGSLConstants.PROGRAM_TYPE_VERTEX);
			optimizer.optimize(fragmentOutput, OGSLConstants.PROGRAM_TYPE_FRAGMENT);
			log("optimized.");
			log("");
			log("optimized vertex agal:");
			log(vertexOutput.print());
			log("");
			log("optimized fragment agal:");
			log(fragmentOutput.print());
			log("");
			log("vertex shader tokens: " + vertexOutput.numLines());
			log("fragment shader tokens: " + fragmentOutput.numLines());
		}

		private function map():void {
			registerMap.setEnvironment(environment);
			log("va: " + JSON.stringify(registerMap.vertexBufferIndexMap));
			log("vc: " + JSON.stringify(registerMap.vertexConstantsIndexMap));
			log("fc: " + JSON.stringify(registerMap.fragmentConstantsIndexMap));
			log("tex: " + JSON.stringify(registerMap.textureIndexMap));
			log("default vc: " + JSON.stringify(registerMap.defaultConstantsDataVertex));
			log("default fc: " + JSON.stringify(registerMap.defaultConstantsDataFragment));
		}

		private function log(text:Object):void {
			if (logFunction != null) logFunction(text.toString());
		}
	}

}

class OGSLConstants {
	public static const MAX_VARYING_REGISTERS:int = 32;
	public static const MAX_VERTEX_TEMPORARY_REGISTERS:int = 64;
	public static const MAX_VERTEX_CONSTANT_REGISTERS:int = 512;
	public static const MAX_VERTEX_ATTRIBUTE_REGISTERS:int = 64;
	public static const MAX_FRAGMENT_TEMPORARY_REGISTERS:int = 64;
	public static const MAX_FRAGMENT_CONSTANT_REGISTERS:int = 128;

	public static const TYPE_FLOAT:String = "float";
	public static const TYPE_VEC2:String = "vec2";
	public static const TYPE_VEC3:String = "vec3";
	public static const TYPE_VEC4:String = "vec4";
	public static const TYPE_MAT3X4:String = "mat3x4";
	public static const TYPE_MAT4X4:String = "mat4x4";

	public static const SCOPE_GLOBAL:String = "global";
	public static const SCOPE_VERTEX:String = "global.vertex";
	public static const SCOPE_FRAGMENT:String = "global.fragment";
	public static const SCOPE_TYPE_GLOBAL:String = "global";
	public static const SCOPE_TYPE_VERTEX:String = "vertex";
	public static const SCOPE_TYPE_FRAGMENT:String = "fragment";

	public static const PROGRAM_TYPE_VERTEX:String = "vertex";
	public static const PROGRAM_TYPE_FRAGMENT:String = "fragment";

	public static const BUILT_IN_MIN:String = "min";
	public static const BUILT_IN_MAX:String = "max";
	public static const BUILT_IN_STEP:String = "step";
	public static const BUILT_IN_LESS_THAN:String = "lessThan";
	public static const BUILT_IN_LESS_THAN_EQUAL:String = "lessThanEqual";
	public static const BUILT_IN_GREATER_THAN:String = "greaterThan";
	public static const BUILT_IN_GREATER_THAN_EQUAL:String = "greaterThanEqual";
	public static const BUILT_IN_EQUAL:String = "equal";
	public static const BUILT_IN_NOT_EQUAL:String = "notEqual";
	public static const BUILT_IN_CLAMP:String = "clamp";
	public static const BUILT_IN_SMOOTHSTEP:String = "smoothstep";
	public static const BUILT_IN_POW:String = "pow";
	public static const BUILT_IN_DOT:String = "dot";
	public static const BUILT_IN_CROSS:String = "cross";
	public static const BUILT_IN_MUL:String = "mul";
	public static const BUILT_IN_DISTANCE:String = "distance";
	public static const BUILT_IN_REFLECT:String = "reflect";
	public static const BUILT_IN_REFRACT:String = "refract";
	public static const BUILT_IN_MOD:String = "mod";
	public static const BUILT_IN_MIX:String = "mix";
	public static const BUILT_IN_LENGTH:String = "length";
	public static const BUILT_IN_NORMALIZE:String = "normalize";
	public static const BUILT_IN_SQRT:String = "sqrt";
	public static const BUILT_IN_RSQRT:String = "rsqrt";
	public static const BUILT_IN_LOG2:String = "log2";
	public static const BUILT_IN_EXP2:String = "exp2";
	public static const BUILT_IN_SIN:String = "sin";
	public static const BUILT_IN_COS:String = "cos";
	public static const BUILT_IN_TAN:String = "tan";
	public static const BUILT_IN_ABS:String = "abs";
	public static const BUILT_IN_SATURATE:String = "saturate";
	public static const BUILT_IN_FRACT:String = "fract";
	public static const BUILT_IN_FLOOR:String = "floor";
	public static const BUILT_IN_CEIL:String = "ceil";
	public static const BUILT_IN_ROUND:String = "round";
	public static const BUILT_IN_TEX_2D:String = "tex2D";
	public static const BUILT_IN_TEX_CUBE:String = "texCube";
	public static const BUILT_IN_VEC2:String = OGSLConstants.TYPE_VEC2;
	public static const BUILT_IN_VEC3:String = OGSLConstants.TYPE_VEC3;
	public static const BUILT_IN_VEC4:String = OGSLConstants.TYPE_VEC4;
	public static const BUILT_IN_MAT3X4:String = OGSLConstants.TYPE_MAT3X4;
	public static const BUILT_IN_MAT4X4:String = OGSLConstants.TYPE_MAT4X4;
}

class OGSLRegisterMap {
	public var vertexBufferIndexMap:Object;
	public var vertexConstantsIndexMap:Object;
	public var fragmentConstantsIndexMap:Object;
	public var textureIndexMap:Object;
	public var defaultConstantsDataVertex:Vector.<Number>; // {index, x, y, z, w} * n
	public var defaultConstantsDataFragment:Vector.<Number>; // {index, x, y, z, w} * n

	public function setEnvironment(env:OGSLEnvironment):void {
		vertexBufferIndexMap = env.getVertexBufferIndexMap();
		vertexConstantsIndexMap = env.getVertexConstantsIndexMap();
		fragmentConstantsIndexMap = env.getFragmentConstantsIndexMap();
		textureIndexMap = env.getTextureIndexMap();
		defaultConstantsDataVertex = env.getDefaultConstantsDataVertex();
		defaultConstantsDataFragment = env.getDefaultConstantsDataFragment();
	}
}

class OGSLToken {
	public static const NUMBER:int     = 0;
	public static const SYSTEM:int     = 1;
	public static const SYMBOL:int     = 2;
	public static const IDENTIFIER:int = 3;
	public static const EOF:int        = 4;
	public var next:OGSLToken;

	public var type:int;
	public var data:String;

	public function OGSLToken(type:int, data:String) {
		if (type < NUMBER || type > EOF) throw new Error("!?");
		this.type = type;
		this.data = data;
	}

	public function toString():String {
		switch (type) {
		case NUMBER:
			return "[num " + data + "]";
		case SYSTEM:
			return "[sys " + data + "]";
		case SYMBOL:
			return "[sym " + data + "]";
		case IDENTIFIER:
			return "[id " + data + "]";
		case EOF:
			return "[EOF]";
		default:
			return "[!!!INVALID TOKEN!!! " + data + "]";
		}
	}
}

class OGSLTokenizer {
	public static const RESERVED_WORDS:Vector.<String> = new <String>[
		"program", OGSLConstants.PROGRAM_TYPE_VERTEX, OGSLConstants.PROGRAM_TYPE_FRAGMENT, "this", "var", "const", "function",
		"void", OGSLConstants.TYPE_FLOAT, OGSLConstants.TYPE_VEC2, OGSLConstants.TYPE_VEC3, OGSLConstants.TYPE_VEC4, OGSLConstants.TYPE_MAT3X4, OGSLConstants.TYPE_MAT4X4,
		"varying", "uniform", "attribute",
		"if", "else",
		"public", "private", "protected", "static", "final", // unused
		"package", "import", "class", "interface", "extends", "implements", // unused
		"super", "int", "boolean", "true", "false", "null", // unused
		"do", "while", "for", "try", "catch", "switch", "continue", "break" // unused
	];
	public static const MULTIBYTE_SYMBOLS:Vector.<String> = new <String>[
		"||", "&&", "==", "!=", "<=", ">=", "+=", "-=", "*=", "/="
	];
	private var lastToken:OGSLToken;
	private var input:StringReader;

	public var tokens:OGSLToken;

	public function OGSLTokenizer() {
	}

	public function tokenize(ogslCode:String):void {
		input = new StringReader(ogslCode);
		readAll();
	}

	private function readAll():void {
		tokens = null;
		while (!input.isEnd()) {
			readToken();
		}
		appendToken(new OGSLToken(OGSLToken.EOF, "EOF"));
	}

	private function appendToken(token:OGSLToken):void {
		if (tokens) lastToken = lastToken.next = token;
		else lastToken = tokens = token;
	}

	private function readToken():void {
		var n1:String = input.next(1);
		var n2:String = input.next(2);
		if (isSpace(n1)) {
			input.read(); // skip spacing char
		} else if (n1 == "/" && (n2 == "/" || n2 == "*")) {
			readComment();
		} else if (isDigit(n1)) {
			readNumber();
		} else if (isLetter(n1)) {
			readName();
		} else {
			readSymbol();
		}
	}

	private function readComment():void {
		input.read(); // read slash
		if (input.next() == "/") { // line comment
			input.read();
			while (true) {
				if (input.isEnd()) return;
				if (input.read() == "\n") return;
			}
		} else if (input.next() == "*") {
			input.read();
			while (true) {
				if (input.isEnd()) return;
				if (input.read() == "*" && input.next() == "/") {
					input.read();
					return;
				}
			}
		} else throw new Error("!?");
	}

	private function readNumber():void {
		var data:String = input.read(); // read first digit
		if (data == "0" && isDigit(input.next())) throw new Error("invalid number format");
		while (isDigit(input.next())) data += input.read();
		if (input.next() == ".") {
			data += input.read();
			while (isDigit(input.next())) data += input.read();
		}
		if (isLetter(input.next()) || input.next() == ".") throw new Error("invalid number format");
		appendToken(new OGSLToken(OGSLToken.NUMBER, data));
	}

	private function readName():void {
		var data:String = input.read(); // read first letter
		while (isLetter(input.next())) data += input.read();
		var system:Boolean = false;
		for (var i:int = 0; !system && i < RESERVED_WORDS.length; i++) system = data == RESERVED_WORDS[i];
		if (system) appendToken(new OGSLToken(OGSLToken.SYSTEM, data));
		else appendToken(new OGSLToken(OGSLToken.IDENTIFIER, data));
	}

	private function readSymbol():void {
		var data:String = input.read();
		while (true) {
			var tmp:String = data + input.next();
			var multibyte:Boolean = false; // can tmp be a multibyte symbol?
			for (var i:int = 0; !multibyte && i < MULTIBYTE_SYMBOLS.length; i++) multibyte = tmp == MULTIBYTE_SYMBOLS[i].substr(0, tmp.length);
			if (multibyte) data += input.read();
			else break;
		}
		appendToken(new OGSLToken(OGSLToken.SYMBOL, data));
	}

	private function isSpace(c:String):Boolean {
		return c == "\t" || c == "\r" || c == "\n" || c == " ";
	}

	private function isDigit(c:String):Boolean {
		var code:Number = c.charCodeAt(0);
		// 0-9
		return code >= 48 && code <= 57;
	}

	private function isLetter(c:String):Boolean {
		var code:Number = c.charCodeAt(0);
		// 0-9, A-Z, a-z, _
		return code >= 48 && code <= 57 || code >= 65 && code <= 90 || code >= 97 && code <= 122 || c == "_";
	}
}

class OGSLParser {
	private static const BINARY_OPERATOR_PRECEDENCES:Vector.<Vector.<String>> = new <Vector.<String>> [
		// lower
		new <String> [
			"||"
		],
		new <String> [
			"&&"
		],
		new <String> [
			"==", "!="
		],
		new <String> [
			"<", ">", "<=", ">="
		],
		new <String> [
			"+", "-"
		],
		new <String> [
			"*", "/"
		]
		// higher
	];

	private var input:TokenReader;
	public var nodes:OGSLMainNode;

	public function OGSLParser() {
	}

	public function parse(tokens:OGSLToken):void {
		input = new TokenReader(tokens);
		parseMain();
	}

	private function parseMain():void {
		nodes = new OGSLMainNode();
		while (!input.isEnd()) {
			if (isNextDefinitionInstruction()) nodes.addDefinition(parseVariableDefinitionInstruction());
			else if (input.isNext("program")) nodes.addProgram(parseProgram());
			else throw new Error("unexpected token: " + input.next());
		}
	}

	private function parseProgram():OGSLProgramNode {
		var node:OGSLProgramNode = new OGSLProgramNode();
		input.read("program");
		node.type = parseProgramType();
		input.read("{");
		while (!input.isNext("}")) {
			if (isNextDefinitionInstruction()) node.addDefinition(parseVariableDefinitionInstruction());
			else if (input.isNext("function")) node.addFunction(parseFunction());
			else throw new Error("unexpected token: " + input.next());
		}
		input.read("}");
		return node;
	}

	private function isNextDefinitionInstruction():Boolean {
		var next:String = input.next();
		return next == "const" || next == "varying" || next == "uniform" || next == "attribute";
	}

	private function parseFunction():OGSLFunctionNode {
		var node:OGSLFunctionNode = new OGSLFunctionNode();
		input.read("function");
		node.name = input.readIdentifier();
		node.args = parseArgumentsDefinition();
		input.read(":");
		node.returnType = parseType(true);
		parseBlock(node);
		node.addReturnStatementIfRequired();
		return node;
	}

	private function parseBlock(func:OGSLFunctionNode):void {
		input.read("{");
		while (!input.isNext("}")) {
			if (input.isNext(";")) input.read(";"); // skip an empty statement
			else parseStatement(func);
		}
		input.read("}");
	}

	private function parseStatement(func:OGSLFunctionNode):void {
		switch (input.next()) {
		case "var":
			func.addStatement(parseVariableDefinitionStatement());
			break;
		case "if":
			parseIfElseStatement(func);
			break;
		case "loop":
			parseLoopStatement(func);
			break;
		case "return":
			func.addStatement(parseReturnStatement());
			break;
		case "discard":
			func.addStatement(parseDiscardStatement());
			break;
		default:
			func.addStatement(parseAssignmentExpressionStatement());
			break;
		}
	}

	private function parseDiscardStatement():OGSLDiscardStatementNode {
		var node:OGSLDiscardStatementNode = new OGSLDiscardStatementNode();
		input.read("discard");
		input.read(";");
		return node;
	}

	private function parseLoopStatement(func:OGSLFunctionNode):void {
		input.read("loop");
		input.read("(");
		var numLoops:int = parseInt(input.readInteger());
		if (numLoops < 1 || numLoops > 255) throw new Error("invalid loop count: " + numLoops);
		input.read(")");
		var loopFunc:OGSLFunctionNode = new OGSLFunctionNode();
		if (input.isNext("{")) parseBlock(loopFunc); // loop block
		else parseStatement(loopFunc); // loop statement
		for (var i:int = 0; i < numLoops; i++) {
			var statement:OGSLStatementNode = loopFunc.statements;
			while (statement) {
				// remove variable definitions in second or later looping...
				if (i > 0 && statement is OGSLVariableDefinitionStatementNode) {
					var v:OGSLVariableDefinitionAssignmentNode = OGSLVariableDefinitionStatementNode(statement).variables;
					while (v) {
						if (v.assignment) func.addStatement(new OGSLRPNStatementNode(v.variable.name + " " + v.assignment.dumpRPN() + " ="));
						v = v.next;
					}
				} else func.addStatement(new OGSLRPNStatementNode(statement.dumpRPN()));
				statement = statement.next;
			}
		}
	}

	private function parseIfElseStatement(func:OGSLFunctionNode):void {
		var node:OGSLIfStatementNode = new OGSLIfStatementNode();
		input.read("if");
		input.read("(");
		node.condition = parseAssignmentExpression();
		input.read(")");
		func.addStatement(node);
		if (input.isNext("{")) parseBlock(func); // if block
		else parseStatement(func); // if statement
		if (input.isNext("else")) {
			input.read("else");
			func.addStatement(new OGSLElseStatementNode());
			if (input.isNext("if")) parseIfElseStatement(func); // else + if
			else if (input.isNext("{")) parseBlock(func); // else block
			else parseStatement(func); // else statement
		}
		func.addStatement(new OGSLEndIfStatementNode());
	}

	private function parseReturnStatement():OGSLReturnStatementNode {
		var node:OGSLReturnStatementNode = new OGSLReturnStatementNode();
		input.read("return");
		if (!input.isNext(";")) node.expression = parseAssignmentExpression();
		input.read(";");
		return node;
	}

	private function parseAssignmentExpressionStatement():OGSLAssignmentExpressionStatementNode {
		var node:OGSLAssignmentExpressionStatementNode = new OGSLAssignmentExpressionStatementNode();
		node.expression = parseAssignmentExpression();
		input.read(";");
		return node;
	}

	private function parseVariableDefinitionStatement():OGSLVariableDefinitionStatementNode {
		var node:OGSLVariableDefinitionStatementNode = new OGSLVariableDefinitionStatementNode();
		input.read("var");
		while (true) {
			node.addVariable(parseVariableDefinitionAssignment());
			if (input.isNext(";")) break;
			else input.read(",");
		}
		input.read(";");
		return node;
	}

	private function parseVariableDefinitionAssignment():OGSLVariableDefinitionAssignmentNode {
		var node:OGSLVariableDefinitionAssignmentNode = new OGSLVariableDefinitionAssignmentNode();
		node.variable = parseVariable();
		if (input.isNext("=")) {
			input.read("=");
			node.assignment = parseAssignmentExpression();
		}
		return node;
	}

	private function parseAssignmentExpression():OGSLExpressionNode {
		var lhs:OGSLExpressionNode = parseBinaryExpression();
		var next:String = input.next();
		if (next == "=" || next == "+=" || next == "-=" || next == "*=" || next == "/=") {
			var node:OGSLAssignmentExpressionNode = new OGSLAssignmentExpressionNode();
			node.lhs = lhs;
			node.operator = input.read();
			node.rhs = parseAssignmentExpression();
			return node;
		}
		return lhs;
	}

	private function parseBinaryExpression():OGSLExpressionNode {
		return parseBinaryExpressionImpl(parseUnaryExpression(), 0);
	}

	private function parseBinaryExpressionImpl(lhs:OGSLExpressionNode, minPrecedence:int):OGSLExpressionNode {
		var precedence:int;
		while ((precedence = nextBinaryOperatorPrecedence()) >= minPrecedence) {
			var operator:String = input.read();
			var rhs:OGSLExpressionNode = parseUnaryExpression();
			if (nextBinaryOperatorPrecedence() > precedence) {
				// parse all operators whose precedence is higher than current one
				rhs = parseBinaryExpressionImpl(rhs, precedence + 1);
			}
			var newLhs:OGSLBinaryExpressionNode = new OGSLBinaryExpressionNode();
			newLhs.lhs = lhs;
			newLhs.operator = operator;
			newLhs.rhs = rhs;
			lhs = newLhs;
		}
		return lhs;
	}

	private function nextBinaryOperatorPrecedence():int {
		var data:String = input.next();
		for (var i:int = 0; i < BINARY_OPERATOR_PRECEDENCES.length; i++) {
			for (var j:int = 0; j < BINARY_OPERATOR_PRECEDENCES[i].length; j++) {
				if (BINARY_OPERATOR_PRECEDENCES[i][j] == data) return i;
			}
		}
		return -1; // not a binary operator
	}

	private function parseUnaryExpression():OGSLExpressionNode {
		if (input.isNext("-")) {
			input.read("-");
			var node:OGSLUnaryExpressionNode = new OGSLUnaryExpressionNode();
			node.operator = "~"; // negate
			node.rhs = parseMemberExpression(parsePrimaryExpression());
			return node;
		}
		return parseMemberExpression(parsePrimaryExpression());
	}

	private function parsePrimaryExpression():OGSLExpressionNode {
		if (input.isNextType(OGSLToken.IDENTIFIER) || isNextCreatableType()) {
			return parseIdentifierExpression(true);
		} else if (input.isNextType(OGSLToken.NUMBER)) {
			return parseLiteralNumberExpression();
		} else if (input.isNext("(")) {
			return parseParenExpression();
		} else if (input.isNext("this")) {
			return parseThisAccessExpression();
		}
		throw new Error("unexpected token: " + input.next());
	}

	private function parseIdentifierExpression(allowCreatableType:Boolean = false):OGSLIdentifierExpressionNode {
		var node:OGSLIdentifierExpressionNode = new OGSLIdentifierExpressionNode();
		if (allowCreatableType && isNextCreatableType()) node.name = input.read();
		else node.name = input.readIdentifier();
		return node;
	}

	private function parseLiteralNumberExpression():OGSLLiteralNumberExpressionNode {
		var node:OGSLLiteralNumberExpressionNode = new OGSLLiteralNumberExpressionNode();
		node.number = input.readNumber();
		return node;
	}

	private function parseThisAccessExpression():OGSLThisAccessExpressionNode {
		var node:OGSLThisAccessExpressionNode = new OGSLThisAccessExpressionNode();
		input.read("this");
		input.read(".");
		node.name = input.readIdentifier();
		return node;
	}

	private function parseParenExpression():OGSLExpressionNode {
		input.read("(");
		var node:OGSLExpressionNode = parseAssignmentExpression();
		input.read(")");
		return node;
	}

	private function parseMemberExpression(lhs:OGSLExpressionNode):OGSLExpressionNode {
		var next:String = input.next();
		if (next == "(") {
			return parseMemberExpression(parseFunctionCallExpression(lhs));
		} else if (next == "." || next == "[") {
			return parseMemberExpression(parseRegisterExpression(lhs));
		}
		return lhs;
	}

	private function parseFunctionCallExpression(lhs:OGSLExpressionNode):OGSLExpressionNode {
		if (!(lhs is OGSLIdentifierExpressionNode)) throw new Error("invalid function call");
		var node:OGSLFunctionCallExpressionNode = new OGSLFunctionCallExpressionNode();
		node.name = OGSLIdentifierExpressionNode(lhs);
		node.args = parseArguments();
		return node;
	}

	private function parseArguments():OGSLArgumentsNode {
		var node:OGSLArgumentsNode = new OGSLArgumentsNode();
		input.read("(");
		if (!input.isNext(")")) {
			while (true) {
				node.addExpression(parseAssignmentExpression());
				if (input.isNext(")")) break;
				else input.read(",");
			}
		}
		input.read(")");
		return node;
	}

	private function parseArgumentsDefinition():OGSLArgumentsDefinitionNode {
		var node:OGSLArgumentsDefinitionNode = new OGSLArgumentsDefinitionNode();
		input.read("(");
		if (!input.isNext(")")) {
			while (true) {
				node.addVariable(parseVariable(false, true));
				if (input.isNext(")")) break;
				else input.read(",");
			}
		}
		input.read(")");
		return node;
	}

	private function parseVariableDefinitionInstruction():OGSLVariableDefinitionInstructionNode {
		var node:OGSLVariableDefinitionInstructionNode = new OGSLVariableDefinitionInstructionNode();
		switch (input.next()) {
		case "varying":
			node.type = input.read("varying");
			break;
		case "attribute":
			node.type = input.read("attribute");
			break;
		case "uniform":
			node.type = input.read("uniform");
			break;
		case "const":
			node.type = input.read("const");
			break;
		default:
			throw new Error("unexpected token: " + input.next());
		}
		if (!input.isNext(";")) {
			while (true) {
				node.addVariable(parseVariable(true));
				if (input.isNext(";")) break;
				else input.read(",");
			}
		}
		input.read(";");
		return node;
	}

	private function parseVariable(allowTexture:Boolean = false, allowPassByReference:Boolean = false):OGSLVariableDefinitionNode {
		var node:OGSLVariableDefinitionNode = new OGSLVariableDefinitionNode();
		if (allowPassByReference && input.isNext("&")) {
			input.read("&");
			node.passByReference = true;
		}
		node.name = input.readIdentifier();
		input.read(":");
		node.type = parseType(false, allowTexture);
		return node;
	}

	private function parseRegisterExpression(lhs:OGSLExpressionNode):OGSLRegisterExpressionNode {
		var node:OGSLRegisterExpressionNode = new OGSLRegisterExpressionNode();
		node.lhs = lhs;
		if (input.isNext("[")) {
			input.read("[");
			node.index = input.readInteger();
			input.read("]");
		} else node.index = "";
		if (input.isNext(".")) {
			input.read(".");
			node.components = parseComponents();
		} else node.components = "";
		if (node.index == "" && node.components == "") throw new Error("!?");
		return node;
	}

	private function parseProgramType():String {
		var type:String = input.read();
		if (type != OGSLConstants.PROGRAM_TYPE_VERTEX && type != OGSLConstants.PROGRAM_TYPE_FRAGMENT) throw new Error("unexpected program type: " + type);
		return type;
	}

	private function parseType(allowVoid:Boolean = false, allowTexture:Boolean = false):String {
		var type:String = input.read();
		if (allowVoid && type == "void") return type;
		if (allowTexture && type == "texture") return type;
		if (type != OGSLConstants.TYPE_FLOAT && type != OGSLConstants.TYPE_VEC2 && type != OGSLConstants.TYPE_VEC3 && type != OGSLConstants.TYPE_VEC4 && type != OGSLConstants.TYPE_MAT3X4 && type != OGSLConstants.TYPE_MAT4X4) throw new Error("invalid type: " + type);
		return type;
	}

	private function isNextCreatableType():Boolean {
		var next:String = input.next();
		return next == OGSLConstants.TYPE_VEC2 || next == OGSLConstants.TYPE_VEC3 || next == OGSLConstants.TYPE_VEC4 || next == OGSLConstants.TYPE_MAT3X4 || next == OGSLConstants.TYPE_MAT4X4;
	}

	private function parseComponents():String {
		var components:String = input.read();
		components = components.replace(/r/g, "x").replace(/g/g, "y").replace(/b/g, "z").replace(/a/g, "w");
		if (components.length > 4 || components.replace(/[xyzw]/g, "") != "") throw new Error("invalid components: " + components);
		return components;
	}
}

class OGSLAnalyzer {
	private var nodes:OGSLMainNode;
	public var env:OGSLEnvironment;

	public function OGSLAnalyzer() {
	}

	public function analyze(nodes:OGSLMainNode):void {
		this.nodes = nodes;
		env = new OGSLEnvironment();
		analyzeMain();
	}

	private function analyzeMain():void {
		analyzeDefinitions(nodes.definitions, OGSLConstants.SCOPE_GLOBAL);
		analyzePrograms(nodes.programs);
		// allocate vertex output variables
		var opDummy:OGSLVariable = new OGSLVariable();
		opDummy.name = "output";
		opDummy.type = OGSLConstants.TYPE_VEC4;
		opDummy.scope = OGSLConstants.SCOPE_VERTEX;
		opDummy.registerName = "vt";
		opDummy.registerComponentOrder = "xyzw";
		opDummy.registerIndex = -1;
		env.addVariable(opDummy);
		var op:OGSLVariable = new OGSLVariable();
		op.name = "%output";
		op.type = OGSLConstants.TYPE_VEC4;
		op.scope = OGSLConstants.SCOPE_VERTEX;
		op.registerName = "op";
		op.registerComponentOrder = "xyzw";
		op.registerIndex = -1;
		env.addVariable(op);
		// allocate fragment output variables
		var ocDummy:OGSLVariable = new OGSLVariable();
		ocDummy.name = "output";
		ocDummy.type = OGSLConstants.TYPE_VEC4;
		ocDummy.scope = OGSLConstants.SCOPE_FRAGMENT;
		ocDummy.registerName = "ft";
		ocDummy.registerComponentOrder = "xyzw";
		ocDummy.registerIndex = -1;
		env.addVariable(ocDummy);
		var oc:OGSLVariable = new OGSLVariable();
		oc.name = "%output";
		oc.type = OGSLConstants.TYPE_VEC4;
		oc.scope = OGSLConstants.SCOPE_FRAGMENT;
		oc.registerName = "oc";
		oc.registerComponentOrder = "xyzw";
		oc.registerIndex = -1;
		env.addVariable(oc);
		//
		addBuiltInFunction(OGSLConstants.BUILT_IN_MIN, 2);
		addBuiltInFunction(OGSLConstants.BUILT_IN_MAX, 2);
		addBuiltInFunction(OGSLConstants.BUILT_IN_STEP, 2);
		addBuiltInFunction(OGSLConstants.BUILT_IN_LESS_THAN, 2);
		addBuiltInFunction(OGSLConstants.BUILT_IN_LESS_THAN_EQUAL, 2);
		addBuiltInFunction(OGSLConstants.BUILT_IN_GREATER_THAN, 2);
		addBuiltInFunction(OGSLConstants.BUILT_IN_GREATER_THAN_EQUAL, 2);
		addBuiltInFunction(OGSLConstants.BUILT_IN_EQUAL, 2);
		addBuiltInFunction(OGSLConstants.BUILT_IN_NOT_EQUAL, 2);
		addBuiltInFunction(OGSLConstants.BUILT_IN_CLAMP, 3);
		addBuiltInFunction(OGSLConstants.BUILT_IN_SMOOTHSTEP, 3);
		addBuiltInFunction(OGSLConstants.BUILT_IN_POW, 2);
		addBuiltInFunction(OGSLConstants.BUILT_IN_DOT, 2);
		addBuiltInFunction(OGSLConstants.BUILT_IN_CROSS, 2);
		addBuiltInFunction(OGSLConstants.BUILT_IN_MUL, 2);
		addBuiltInFunction(OGSLConstants.BUILT_IN_DISTANCE, 2);
		addBuiltInFunction(OGSLConstants.BUILT_IN_REFLECT, 2);
		addBuiltInFunction(OGSLConstants.BUILT_IN_REFRACT, 3);
		addBuiltInFunction(OGSLConstants.BUILT_IN_MOD, 2);
		addBuiltInFunction(OGSLConstants.BUILT_IN_MIX, 3);
		addBuiltInFunction(OGSLConstants.BUILT_IN_LENGTH, 1);
		addBuiltInFunction(OGSLConstants.BUILT_IN_NORMALIZE, 1);
		addBuiltInFunction(OGSLConstants.BUILT_IN_SQRT, 1);
		addBuiltInFunction(OGSLConstants.BUILT_IN_RSQRT, 1);
		addBuiltInFunction(OGSLConstants.BUILT_IN_LOG2, 1);
		addBuiltInFunction(OGSLConstants.BUILT_IN_EXP2, 1);
		addBuiltInFunction(OGSLConstants.BUILT_IN_SIN, 1);
		addBuiltInFunction(OGSLConstants.BUILT_IN_COS, 1);
		addBuiltInFunction(OGSLConstants.BUILT_IN_TAN, 1);
		addBuiltInFunction(OGSLConstants.BUILT_IN_ABS, 1);
		addBuiltInFunction(OGSLConstants.BUILT_IN_SATURATE, 1);
		addBuiltInFunction(OGSLConstants.BUILT_IN_FRACT, 1);
		addBuiltInFunction(OGSLConstants.BUILT_IN_FLOOR, 1);
		addBuiltInFunction(OGSLConstants.BUILT_IN_CEIL, 1);
		addBuiltInFunction(OGSLConstants.BUILT_IN_ROUND, 1);
		addBuiltInFunction(OGSLConstants.BUILT_IN_TEX_2D, -1);
		addBuiltInFunction(OGSLConstants.BUILT_IN_TEX_CUBE, -1);
		addBuiltInFunction(OGSLConstants.BUILT_IN_VEC2, -1);
		addBuiltInFunction(OGSLConstants.BUILT_IN_VEC3, -1);
		addBuiltInFunction(OGSLConstants.BUILT_IN_VEC4, -1);
		addBuiltInFunction(OGSLConstants.BUILT_IN_MAT3X4, -1);
		addBuiltInFunction(OGSLConstants.BUILT_IN_MAT4X4, -1);
	}

	private function addBuiltInFunction(name:String, numArgs:int):void {
		var f:OGSLFunction = new OGSLFunction();
		f.scope = OGSLConstants.SCOPE_GLOBAL;
		f.name = name;
		f.args = numArgs == -1 ? "*" : String(numArgs);
		f.type = "function";
		f.rpn = "@builtin";
		f.returnType = "";
		env.addFunction(f);
	}

	private function analyzePrograms(programs:OGSLProgramNode):void {
		var hasVertexProgram:Boolean = false;
		var hasFragmentProgram:Boolean = false;
		while (programs) {
			if (programs.type == OGSLConstants.PROGRAM_TYPE_VERTEX) {
				if (hasVertexProgram) throw new Error("error: duplicate vertex program");
				hasVertexProgram = true;
			} else if (programs.type == OGSLConstants.PROGRAM_TYPE_FRAGMENT) {
				if (hasFragmentProgram) throw new Error("error: duplicate fragment program");
				hasFragmentProgram = true;
			} else throw new Error("!?");
			var scope:String = env.getScopeByProgramType(programs.type);
			analyzeDefinitions(programs.definitions, scope);
			analyzeFunctions(programs.functions, scope);
			var main:OGSLFunction = env.accessFunction(new OGSLAccessor("main", scope));
			if (main == null) throw new Error("programs must have main function");
			if (main.args != "") throw new Error("main function must not have arguments");
			main.complementOutput();
			programs = programs.next;
		}
		if (!hasVertexProgram || !hasFragmentProgram) throw new Error("there must be both vertex and fragment programs");
	}

	private function analyzeDefinitions(definition:OGSLVariableDefinitionInstructionNode, scope:String):void {
		var texIndex:int = 0;
		while (definition) {
			var vars:OGSLVariableDefinitionNode = definition.variables;
			var v:OGSLVariable;
			var t:OGSLTexture;
			var registerName:String;
			switch (scope) {
			case OGSLConstants.SCOPE_GLOBAL:
				switch (definition.type) {
				case "varying":
					while (vars) {
						if (vars.type == "texture") throw new Error("you cannot define textures in global scope");
						v = new OGSLVariable();
						v.scope = scope;
						v.name = vars.name;
						v.type = vars.type;
						v.registerName = "v";
						env.addVariable(v);
						vars = vars.next;
					}
					break;
				case "uniform":
					throw new Error("you cannot define uniform variables in global scope");
				case "attribute":
					throw new Error("you cannot define attribute variables in global scope");
				case "const":
					throw new Error("constants are not supported yet"); // TODO: implement constants
					break;
				default:
					throw new Error("!?");
				}
				break;
			case OGSLConstants.SCOPE_VERTEX:
				switch (definition.type) {
				case "varying":
					throw new Error("you cannot define varying variables in vertex program scope");
				case "uniform":
					registerName = "vc";
					break;
				case "attribute":
					registerName = "va";
					break;
				case "const":
					throw new Error("constants are not supported yet"); // TODO: implement constants
				default:
					throw new Error("!?");
				}
				while (vars) {
					if (vars.type == "texture") throw new Error("you cannot define textures in vertex program scope");
					v = new OGSLVariable();
					v.scope = scope;
					v.name = vars.name;
					v.type = vars.type;
					v.registerName = registerName;
					env.addVariable(v);
					vars = vars.next;
				}
				break;
			case OGSLConstants.SCOPE_FRAGMENT:
				switch (definition.type) {
				case "varying":
					throw new Error("you cannot define varying variables in fragment program scope");
				case "uniform":
					registerName = "fc";
					break;
				case "attribute":
					throw new Error("you cannot define attribute variables in fragment program scope");
				case "const":
					throw new Error("constants are not supported yet"); // TODO: implement constants
				default:
					throw new Error("!?");
				}
				while (vars) {
					if (vars.type == "texture") {
						t = new OGSLTexture();
						t.name = vars.name;
						t.scope = scope;
						t.type = vars.type;
						t.textureIndex = texIndex++;
						env.addTexture(t);
					} else {
						v = new OGSLVariable();
						v.scope = scope;
						v.name = vars.name;
						v.type = vars.type;
						v.registerName = "fc";
						env.addVariable(v);
					}
					vars = vars.next;
				}
				break;
			}
			definition = definition.next;
		}
	}

	private function analyzeFunctions(func:OGSLFunctionNode, scope:String):void {
		while (func) {
			var args:String = "";
			var variables:OGSLVariableDefinitionNode = func.args.variables;
			while (variables) {
				if (args != "") args += ",";
				if (variables.passByReference) args += "&";
				args += variables.name + ":" + variables.type;
				variables = variables.next;
			}
			var f:OGSLFunction = new OGSLFunction();
			f.scope = scope;
			f.name = func.name;
			f.type = "function";
			f.returnType = func.returnType;
			f.args = args;
			f.rpn = func.dumpRPN();
			env.addFunction(f);
			func = func.next;
		}
	}
}

class OGSLEmitter {
	private var env:OGSLEnvironment;
	private var stack:Vector.<OGSLAccessor>;
	private var stackCount:int;
	private var baseStackCount:int;
	public var vertexOutput:OGSLOutput;
	public var fragmentOutput:OGSLOutput;
	private var emitMode:int; // 0:vertex, 1:fragment

	public function OGSLEmitter() {
		stack = new Vector.<OGSLAccessor>(128, true);
		for (var i:int = 0; i < stack.length; i++) {
			stack[i] = new OGSLAccessor();
		}
	}

	public function emit(env:OGSLEnvironment):void {
		this.env = env;
		run();
	}

	private function run():void {
		// -------- run vertex program --------
		emitMode = 0;
		vertexOutput = new OGSLOutput();
		runProgram(OGSLConstants.PROGRAM_TYPE_VERTEX);
		// -------- run fragment program --------
		emitMode = 1;
		fragmentOutput = new OGSLOutput();
		runProgram(OGSLConstants.PROGRAM_TYPE_FRAGMENT);
	}

	private function runProgram(type:String):void {
		stackCount = 0;
		baseStackCount = 0;
		push("main", env.getScopeByProgramType(type));
		push("0");
		call(); // run main function
	}

	private function runRPN(rpn:String, scope:String, returnTo:OGSLAccessor):void {
		var data:Vector.<String> = Vector.<String>(rpn.split(" "));
		data.fixed = true;
		for (var i:int = 0; i < data.length; i++) {
			var op:String = data[i];
			switch (op) {
			case ";": // a statement finished
				endStatement(scope);
				break;
			case "var": // variable definition
				defineVariable();
				break;
			case "()": // function call
				call();
				break;
			case "return":
				doReturn(returnTo);
				break;
			case "this":
				thisAccess();
				break;
			case ".": // register access
				access();
				break;
			case "=":
				assign();
				break;
			case "+=":
				binaryOpEqual("add");
				break;
			case "-=":
				binaryOpEqual("sub");
				break;
			case "*=":
				binaryOpEqual("mul");
				break;
			case "/=":
				binaryOpEqual("div");
				break;
			case "+":
				binaryOp("add");
				break;
			case "-":
				binaryOp("sub");
				break;
			case "*":
				binaryOp("mul");
				break;
			case "/":
				binaryOp("div");
				break;
			case "~":
				unaryOp("neg");
				break;
			case "==":
				binaryOp("seq", OGSLConstants.TYPE_FLOAT);
				break;
			case "!=":
				binaryOp("sne", OGSLConstants.TYPE_FLOAT);
				break;
			case "<":
				binaryOp("slt", OGSLConstants.TYPE_FLOAT, "", false);
				break;
			case ">":
				binaryOp("slt", OGSLConstants.TYPE_FLOAT, "", true);
				break;
			case "<=":
				binaryOp("sge", OGSLConstants.TYPE_FLOAT, "", true);
				break;
			case ">=":
				binaryOp("sge", OGSLConstants.TYPE_FLOAT, "", false);
				break;
			case "||":
				binaryOp("add", OGSLConstants.TYPE_FLOAT);
				break;
			case "&&":
				binaryOp("mul", OGSLConstants.TYPE_FLOAT);
				break;
			case "if":
				beginIf();
				break;
			case "else":
				beginElse();
				break;
			case "endif":
				endIf();
				break;
			case "discard":
				discard(scope);
				break;
			default:
				push(op, scope);
				break;
			}
		}
	}

	private function beginIf():void {
		var condition:OGSLAccessor = pop();
		if (env.getVariableType(condition) != OGSLConstants.TYPE_FLOAT) throw new Error("invalid if-else statement use");
		if (env.isConstant(condition)) {
			pusha(env.createTempVariable(OGSLConstants.TYPE_FLOAT, condition.scope));
			pusha(condition);
			assign();
			condition = pop();
		}
		agalop("ine", null, fld(condition), fld(new OGSLAccessor("0", condition.scope))); // condition != 0
	}

	private function beginElse():void {
		agalop("els");
	}

	private function endIf():void {
		agalop("eif");
	}

	private function discard(scope:String):void {
		pusha(env.createTempVariable(OGSLConstants.TYPE_FLOAT, scope));
		push("-1", scope);
		assign();
		agalop("kil", null, fld(pop()));
	}

	private function endStatement(scope:String):void {
		stackCount = baseStackCount; // clear the stack
		env.destroyTempVariablesAt(scope);
	}

	private function defineVariable():void {
		var type:String = pop().name;
		var tmp:OGSLAccessor = pop();
		var v:OGSLVariable = new OGSLVariable();
		v.type = type;
		v.name = tmp.name;
		v.scope = tmp.scope;
		switch (env.getScopeType(tmp.scope)) {
		case OGSLConstants.SCOPE_TYPE_VERTEX:
			v.registerName = "vt";
			break;
		case OGSLConstants.SCOPE_TYPE_FRAGMENT:
			v.registerName = "ft";
			break;
		default:
			throw new Error("!?");
		}
		env.addVariable(v);
	}

	private function call():void {
		var numArgs:int = parseInt(pop().name);
		var functionAccessor:OGSLAccessor = pop();
		var func:OGSLFunction = env.accessFunction(functionAccessor);
		if (func == null) throw new Error("no such function: " + functionAccessor.name);
		if (func.rpn == "@builtin") {
			if (func.args != "*" && numArgs != parseInt(func.args)) throw new Error("incorrect number of arguments in calling function " + func.name + ": expected " + parseInt(func.args) + " but given " + numArgs);
			callBuiltInFunction(func.name, numArgs, functionAccessor.scope);
			return;
		}
		var argVariables:Array = func.args == "" ? new Array() : func.args.split(","); // arg1,arg2,...,argN
		if (numArgs != argVariables.length) throw new Error("incorrect number of arguments in calling function " + func.name + ": expected " + argVariables.length + " but given " + numArgs);

		var scope:String = functionAccessor.scope;
		// check recursive function call
		if (scope.match(new RegExp("->" + func.name + "($|->)")) != null) throw new Error("recursive function call is found: " + func.name);

		// note that the order of stacked values of arguments is reversed
		var originalArgAccessors:Vector.<OGSLAccessor> = new Vector.<OGSLAccessor>(numArgs, true);
		for (var i:int = numArgs - 1; i >= 0; i--) {
			originalArgAccessors[i] = pop();
		}

		var returnType:String = func.returnType;

		var returnTo:OGSLAccessor = null;
		if (returnType == "void") push("void"); // causes an error when accessed
		else {
			returnTo = env.createTempVariable(returnType, scope);
			pusha(returnTo); // push returned data
		}

		// push data
		var oldBaseStackCount:int = baseStackCount;
		baseStackCount = stackCount;

		// update data
		if (scope == OGSLConstants.SCOPE_GLOBAL) throw new Error("!?");
		if (scope == OGSLConstants.SCOPE_VERTEX || scope == OGSLConstants.SCOPE_FRAGMENT) {
			if (func.name != "main") throw new Error("!?");
			scope += "." + func.name;
		} else {
			if (func.name == "main") throw new Error("you cannot call main function");
			scope += "->" + func.name;
		}

		// define argument variables
		var copiedArgAccessors:Vector.<OGSLAccessor> = new Vector.<OGSLAccessor>(numArgs, true);
		for (i = 0; i < numArgs; i++) {
			var argVariable:Array = argVariables[i].split(":");
			var argName:String = argVariable[0];
			var argType:String = argVariable[1];
			var enablePassByReference:Boolean = argName.indexOf("&") == 0;
			if (enablePassByReference) argName = argName.substring(1);
			// define
			push(argName, scope);
			push(argType);
			defineVariable();
			// assign
			push(argName, scope);
			pusha(originalArgAccessors[i]);
			assign(enablePassByReference);
			copiedArgAccessors[i] = enablePassByReference ? pop() : null;
		}

		// run function rpn
		runRPN(func.rpn, scope, returnTo);

		// copy modified data
		for (i = 0; i < numArgs; i++) {
			if (copiedArgAccessors[i] != null) { // pass by reference is enabled
				pusha(originalArgAccessors[i]);
				pusha(copiedArgAccessors[i]);
				assign(false);
			}
		}

		// destroy local variables
		env.destroyVariablesAt(scope);

		// pop data
		baseStackCount = oldBaseStackCount;
	}

	private function callBuiltInFunction(name:String, numArgs:int, scope:String):void {
		switch (name) {
		case OGSLConstants.BUILT_IN_MIN:
			binaryOp("min");
			return;
		case OGSLConstants.BUILT_IN_MAX:
			binaryOp("max");
			return;
		case OGSLConstants.BUILT_IN_STEP:
			binaryOp("sge", "", "", true);
			return;
		case OGSLConstants.BUILT_IN_LESS_THAN:
			binaryOp("slt");
			return;
		case OGSLConstants.BUILT_IN_LESS_THAN_EQUAL:
			binaryOp("sge", "", "", true);
			return;
		case OGSLConstants.BUILT_IN_GREATER_THAN:
			binaryOp("slt", "", "", true);
			return;
		case OGSLConstants.BUILT_IN_GREATER_THAN_EQUAL:
			binaryOp("sge");
			return;
		case OGSLConstants.BUILT_IN_EQUAL:
			binaryOp("seq");
			return;
		case OGSLConstants.BUILT_IN_NOT_EQUAL:
			binaryOp("sne");
			return;
		case OGSLConstants.BUILT_IN_CLAMP:
			clamp();
			return;
		case OGSLConstants.BUILT_IN_SMOOTHSTEP:
			smoothstep();
			return;
		case OGSLConstants.BUILT_IN_POW:
			binaryOp("pow");
			return;
		case OGSLConstants.BUILT_IN_DOT:
			dot();
			return;
		case OGSLConstants.BUILT_IN_CROSS:
			cross();
			return;
		case OGSLConstants.BUILT_IN_MUL:
			mulMat();
			return;
		case OGSLConstants.BUILT_IN_DISTANCE:
			distance();
			return;
		case OGSLConstants.BUILT_IN_REFLECT:
			reflect();
			return;
		case OGSLConstants.BUILT_IN_REFRACT:
			refract();
			return;
		case OGSLConstants.BUILT_IN_MOD:
			mod();
			return;
		case OGSLConstants.BUILT_IN_MIX:
			mix();
			return;
		case OGSLConstants.BUILT_IN_LENGTH:
			length();
			return;
		case OGSLConstants.BUILT_IN_NORMALIZE:
			normalize();
			return;
		case OGSLConstants.BUILT_IN_SQRT:
			unaryOp("sqt");
			return;
		case OGSLConstants.BUILT_IN_RSQRT:
			unaryOp("rsq");
			return;
		case OGSLConstants.BUILT_IN_LOG2:
			unaryOp("log");
			return;
		case OGSLConstants.BUILT_IN_EXP2:
			unaryOp("exp");
			return;
		case OGSLConstants.BUILT_IN_SIN:
			unaryOp("sin");
			return;
		case OGSLConstants.BUILT_IN_COS:
			unaryOp("cos");
			return;
		case OGSLConstants.BUILT_IN_TAN:
			tan();
			return;
		case OGSLConstants.BUILT_IN_ABS:
			unaryOp("abs");
			return;
		case OGSLConstants.BUILT_IN_SATURATE:
			unaryOp("sat");
			return;
		case OGSLConstants.BUILT_IN_FRACT:
			unaryOp("frc");
			return;
		case OGSLConstants.BUILT_IN_FLOOR:
			floor();
			return;
		case OGSLConstants.BUILT_IN_CEIL:
			ceil();
			return;
		case OGSLConstants.BUILT_IN_ROUND:
			round();
			return;
		case OGSLConstants.BUILT_IN_TEX_2D:
			sample("2d", numArgs);
			return;
		case OGSLConstants.BUILT_IN_TEX_CUBE:
			sample("cube", numArgs);
			return;
		case OGSLConstants.BUILT_IN_VEC2:
			createVec2(numArgs);
			return;
		case OGSLConstants.BUILT_IN_VEC3:
			createVec3(numArgs);
			return;
		case OGSLConstants.BUILT_IN_VEC4:
			createVec4(numArgs);
			return;
		case OGSLConstants.BUILT_IN_MAT3X4:
			createMat3x4(numArgs);
			return;
		case OGSLConstants.BUILT_IN_MAT4X4:
			createMat4x4(numArgs);
			return;
		}
		throw new Error("!?");
	}

	private function tan():void {
		var rad:OGSLAccessor = pop();
		pusha(rad);
		unaryOp("sin");
		pusha(rad);
		unaryOp("cos");
		binaryOp("div");
	}

	private function dot():void {
		var rhs:OGSLAccessor = pop();
		var lhs:OGSLAccessor = pop();
		var typeL:String = env.getVariableType(lhs);
		var typeR:String = env.getVariableType(rhs);
		var tmp:OGSLAccessor;
		if (typeL != typeR) throw new Error("types mismatch: " + typeL + " and " + typeR);
		pusha(lhs);
		pusha(rhs);
		switch (typeL) {
		case OGSLConstants.TYPE_FLOAT:
			binaryOp("mul");
			break;
		case OGSLConstants.TYPE_VEC2:
			binaryOp("mul");
			tmp = pop();
			tmp.components = "x";
			pusha(tmp);
			tmp.components = "y";
			pusha(tmp);
			binaryOp("add");
			break;
		case OGSLConstants.TYPE_VEC3:
			binaryOp("dp3", OGSLConstants.TYPE_VEC3, OGSLConstants.TYPE_FLOAT);
			break;
		case OGSLConstants.TYPE_VEC4:
			binaryOp("dp4", OGSLConstants.TYPE_VEC4, OGSLConstants.TYPE_FLOAT);
			break;
		default:
			throw new Error("unexpected argument type: " + typeL);
		}
	}

	private function cross():void {
		var rhs:OGSLAccessor = pop();
		var lhs:OGSLAccessor = pop();
		pusha(env.createTempVariable(OGSLConstants.TYPE_VEC3, lhs.scope));
		pusha(lhs);
		pusha(rhs);
		binaryOp("crs", OGSLConstants.TYPE_VEC3);
		assign();
	}

	private function createVec2(numArgs:int):void {
		if (numArgs != 2) throw new Error("invalid number of arguments in creating vec2: " + numArgs);
		// float, float
		var a2:OGSLAccessor = pop();
		var a1:OGSLAccessor = pop();
		var type1:String = env.getVariableType(a1);
		var type2:String = env.getVariableType(a2);
		var tmp:OGSLAccessor = env.createTempVariable(OGSLConstants.TYPE_VEC2, a1.scope);
		if (type1 != OGSLConstants.TYPE_FLOAT || type2 != OGSLConstants.TYPE_FLOAT) throw new Error("invalid argument types");
		tmp.components = "x";
		pusha(tmp);
		pusha(a1);
		assign(false);
		tmp.components = "y";
		pusha(tmp);
		pusha(a2);
		assign(false);
		tmp.components = "";
		pusha(tmp);
	}

	private function createVec3(numArgs:int):void {
		var a1:OGSLAccessor, a2:OGSLAccessor, a3:OGSLAccessor;
		var type1:String, type2:String, type3:String;
		var tmp:OGSLAccessor;
		switch (numArgs) {
		case 2:
			// float, vec2 or
			// vec2, float
			a2 = pop();
			a1 = pop();
			type1 = env.getVariableType(a1);
			type2 = env.getVariableType(a2);
			tmp = env.createTempVariable(OGSLConstants.TYPE_VEC3, a1.scope);
			if (type1 == OGSLConstants.TYPE_FLOAT && type2 == OGSLConstants.TYPE_VEC2) {
				tmp.components = "x";
				pusha(tmp);
				pusha(a1);
				assign(false);
				tmp.components = "yz";
				pusha(tmp);
				pusha(a2);
				assign(false);
			} else if (type1 == OGSLConstants.TYPE_VEC2 && type2 == OGSLConstants.TYPE_FLOAT) {
				tmp.components = "xy";
				pusha(tmp);
				pusha(a1);
				assign(false);
				tmp.components = "z";
				pusha(tmp);
				pusha(a2);
				assign(false);
			} else throw new Error("invalid argument types");
			tmp.components = "";
			pusha(tmp);
			break;
		case 3:
			// float, float, float
			a3 = pop();
			a2 = pop();
			a1 = pop();
			type1 = env.getVariableType(a1);
			type2 = env.getVariableType(a2);
			type3 = env.getVariableType(a3);
			tmp = env.createTempVariable(OGSLConstants.TYPE_VEC3, a1.scope);
			if (type1 != OGSLConstants.TYPE_FLOAT || type2 != OGSLConstants.TYPE_FLOAT || type3 != OGSLConstants.TYPE_FLOAT) throw new Error("invalid argument types");
			tmp.components = "x";
			pusha(tmp);
			pusha(a1);
			assign(false);
			tmp.components = "y";
			pusha(tmp);
			pusha(a2);
			assign(false);
			tmp.components = "z";
			pusha(tmp);
			pusha(a3);
			assign(false);
			tmp.components = "";
			pusha(tmp);
			break;
		default:
			throw new Error("invalid number of arguments in creating vec3: " + numArgs);
		}
	}

	private function createVec4(numArgs:int):void {
		var a1:OGSLAccessor, a2:OGSLAccessor, a3:OGSLAccessor, a4:OGSLAccessor;
		var type1:String, type2:String, type3:String, type4:String;
		var tmp:OGSLAccessor;
		switch (numArgs) {
		case 2:
			// float, vec3 or
			// vec2, vec2 or
			// vec3, float
			a2 = pop();
			a1 = pop();
			type1 = env.getVariableType(a1);
			type2 = env.getVariableType(a2);
			tmp = env.createTempVariable(OGSLConstants.TYPE_VEC4, a1.scope);
			if (type1 == OGSLConstants.TYPE_FLOAT && type2 == OGSLConstants.TYPE_VEC3) {
				tmp.components = "x";
				pusha(tmp);
				pusha(a1);
				assign(false);
				tmp.components = "yzw";
				pusha(tmp);
				pusha(a2);
				assign(false);
			} else if (type1 == OGSLConstants.TYPE_VEC2 && type2 == OGSLConstants.TYPE_VEC2) {
				tmp.components = "xy";
				pusha(tmp);
				pusha(a1);
				assign(false);
				tmp.components = "zw";
				pusha(tmp);
				pusha(a2);
				assign(false);
			} else if (type1 == OGSLConstants.TYPE_VEC3 && type2 == OGSLConstants.TYPE_FLOAT) {
				tmp.components = "xyz";
				pusha(tmp);
				pusha(a1);
				assign(false);
				tmp.components = "w";
				pusha(tmp);
				pusha(a2);
				assign(false);
			} else throw new Error("invalid argument types");
			tmp.components = "";
			pusha(tmp);
			break;
		case 3:
			// float, float, vec2 or
			// float, vec2, float or
			// vec2, float, float
			a3 = pop();
			a2 = pop();
			a1 = pop();
			type1 = env.getVariableType(a1);
			type2 = env.getVariableType(a2);
			type3 = env.getVariableType(a3);
			tmp = env.createTempVariable(OGSLConstants.TYPE_VEC4, a1.scope);
			if (type1 == OGSLConstants.TYPE_FLOAT && type2 == OGSLConstants.TYPE_FLOAT && type3 == OGSLConstants.TYPE_VEC2) {
				tmp.components = "x";
				pusha(tmp);
				pusha(a1);
				assign(false);
				tmp.components = "y";
				pusha(tmp);
				pusha(a2);
				assign(false);
				tmp.components = "zw";
				pusha(tmp);
				pusha(a3);
				assign(false);
			} else if (type1 == OGSLConstants.TYPE_FLOAT && type2 == OGSLConstants.TYPE_VEC2 && type3 == OGSLConstants.TYPE_FLOAT) {
				tmp.components = "x";
				pusha(tmp);
				pusha(a1);
				assign(false);
				tmp.components = "yz";
				pusha(tmp);
				pusha(a2);
				assign(false);
				tmp.components = "w";
				pusha(tmp);
				pusha(a3);
				assign(false);
			} else if (type1 == OGSLConstants.TYPE_VEC2 && type2 == OGSLConstants.TYPE_FLOAT && type3 == OGSLConstants.TYPE_FLOAT) {
				tmp.components = "xy";
				pusha(tmp);
				pusha(a1);
				assign(false);
				tmp.components = "z";
				pusha(tmp);
				pusha(a2);
				assign(false);
				tmp.components = "w";
				pusha(tmp);
				pusha(a3);
				assign(false);
			} else throw new Error("invalid argument types");
			tmp.components = "";
			pusha(tmp);
			break;
		case 4:
			// float, float, float, float
			a4 = pop();
			a3 = pop();
			a2 = pop();
			a1 = pop();
			type1 = env.getVariableType(a1);
			type2 = env.getVariableType(a2);
			type3 = env.getVariableType(a3);
			type4 = env.getVariableType(a4);
			tmp = env.createTempVariable(OGSLConstants.TYPE_VEC4, a1.scope);
			if (type1 != OGSLConstants.TYPE_FLOAT || type2 != OGSLConstants.TYPE_FLOAT || type3 != OGSLConstants.TYPE_FLOAT || type4 != OGSLConstants.TYPE_FLOAT) throw new Error("invalid argument types");
			tmp.components = "x";
			pusha(tmp);
			pusha(a1);
			assign(false);
			tmp.components = "y";
			pusha(tmp);
			pusha(a2);
			assign(false);
			tmp.components = "z";
			pusha(tmp);
			pusha(a3);
			assign(false);
			tmp.components = "w";
			pusha(tmp);
			pusha(a4);
			assign(false);
			tmp.components = "";
			pusha(tmp);
			break;
		default:
			throw new Error("invalid number of arguments in creating vec4: " + numArgs);
		}
	}

	private function createMat3x4(numArgs:int):void {
		var a1:OGSLAccessor, a2:OGSLAccessor, a3:OGSLAccessor;
		var type1:String, type2:String, type3:String;
		var tmp:OGSLAccessor;
		switch (numArgs) {
		case 1:
			// float 0 0 0
			// 0 float 0 0
			// 0 0 float 0
			a1 = pop();
			type1 = env.getVariableType(a1);
			tmp = env.createTempVariable(OGSLConstants.TYPE_MAT3X4, a1.scope);
			if (type1 != OGSLConstants.TYPE_FLOAT) throw new Error("invalid argument types");
			tmp.index = "0";
			pusha(tmp);
			pusha(a1);
			push("0", a1.scope);
			push("0", a1.scope);
			push("0", a1.scope);
			createVec4(4);
			assign(false);
			tmp.index = "1";
			pusha(tmp);
			push("0", a1.scope);
			pusha(a1);
			push("0", a1.scope);
			push("0", a1.scope);
			createVec4(4);
			assign(false);
			tmp.index = "2";
			pusha(tmp);
			push("0", a1.scope);
			push("0", a1.scope);
			pusha(a1);
			push("0", a1.scope);
			createVec4(4);
			assign(false);
			tmp.index = "";
			pusha(tmp);
			break;
		case 3:
			// vec4
			// vec4
			// vec4
			a3 = pop();
			a2 = pop();
			a1 = pop();
			type1 = env.getVariableType(a1);
			type2 = env.getVariableType(a2);
			type3 = env.getVariableType(a3);
			tmp = env.createTempVariable(OGSLConstants.TYPE_MAT3X4, a1.scope);
			if (type1 != OGSLConstants.TYPE_VEC4 || type2 != OGSLConstants.TYPE_VEC4 || type3 != OGSLConstants.TYPE_VEC4) throw new Error("invalid argument types");
			tmp.index = "0";
			pusha(tmp);
			pusha(a1);
			assign(false);
			tmp.index = "1";
			pusha(tmp);
			pusha(a2);
			assign(false);
			tmp.index = "2";
			pusha(tmp);
			pusha(a3);
			assign(false);
			tmp.index = "";
			pusha(tmp);
			break;
		default:
			throw new Error("invalid number of arguments in creating mat3x4: " + numArgs);
		}
	}

	private function createMat4x4(numArgs:int):void {
		var a1:OGSLAccessor, a2:OGSLAccessor, a3:OGSLAccessor, a4:OGSLAccessor;
		var type1:String, type2:String, type3:String, type4:String;
		var tmp:OGSLAccessor;
		switch (numArgs) {
		case 1:
			// float 0 0 0
			// 0 float 0 0
			// 0 0 float 0
			// 0 0 0 float
			a1 = pop();
			type1 = env.getVariableType(a1);
			tmp = env.createTempVariable(OGSLConstants.TYPE_MAT4X4, a1.scope);
			if (type1 != OGSLConstants.TYPE_FLOAT) throw new Error("invalid argument types");
			tmp.index = "0";
			pusha(tmp);
			pusha(a1);
			push("0", a1.scope);
			push("0", a1.scope);
			push("0", a1.scope);
			createVec4(4);
			assign(false);
			tmp.index = "1";
			pusha(tmp);
			push("0", a1.scope);
			pusha(a1);
			push("0", a1.scope);
			push("0", a1.scope);
			createVec4(4);
			assign(false);
			tmp.index = "2";
			pusha(tmp);
			push("0", a1.scope);
			push("0", a1.scope);
			pusha(a1);
			push("0", a1.scope);
			createVec4(4);
			assign(false);
			tmp.index = "3";
			pusha(tmp);
			push("0", a1.scope);
			push("0", a1.scope);
			push("0", a1.scope);
			pusha(a1);
			createVec4(4);
			assign(false);
			tmp.index = "";
			pusha(tmp);
			break;
		case 4:
			// vec4
			// vec4
			// vec4
			// vec4
			a4 = pop();
			a3 = pop();
			a2 = pop();
			a1 = pop();
			type1 = env.getVariableType(a1);
			type2 = env.getVariableType(a2);
			type3 = env.getVariableType(a3);
			type4 = env.getVariableType(a4);
			tmp = env.createTempVariable(OGSLConstants.TYPE_MAT4X4, a1.scope);
			if (type1 != OGSLConstants.TYPE_VEC4 || type2 != OGSLConstants.TYPE_VEC4 || type3 != OGSLConstants.TYPE_VEC4 || type4 != OGSLConstants.TYPE_VEC4) throw new Error("invalid argument types");
			tmp.index = "0";
			pusha(tmp);
			pusha(a1);
			assign(false);
			tmp.index = "1";
			pusha(tmp);
			pusha(a2);
			assign(false);
			tmp.index = "2";
			pusha(tmp);
			pusha(a3);
			assign(false);
			tmp.index = "3";
			pusha(tmp);
			pusha(a4);
			assign(false);
			tmp.index = "";
			pusha(tmp);
			break;
		default:
			throw new Error("invalid number of arguments in creating mat4x4: " + numArgs);
		}
	}

	private function sample(type:String, numArgs:int):void {
		var filtering:String = "";
		var mip:String = "";
		var repeat:String = "";
		for (var i:int = 0; i < numArgs - 2; i++) {
			var param:String = pop().name;
			switch (param) {
			case "nearest":
			case "linear":
			case "anisotropic2x":
			case "anisotropic4x":
			case "anisotropic8x":
			case "anisotropic16x":
				if (filtering != "") throw new Error("duplicate filtering flags");
				filtering = param;
				break;
			case "mipnone":
			case "nomip":
			case "mipnearest":
			case "miplinear":
				if (mip != "") throw new Error("duplicate mipmap flags");
				mip = param;
				break;
			case "repeat":
			case "clamp":
			case "repeat_u_clamp_v":
			case "clamp_u_repeat_v":
				if (repeat != "") throw new Error("duplicate repeating flags");
				repeat = param;
				break;
			default:
				throw new Error("invalid sampling flags: " + param);
			}
		}
		if (filtering == "") filtering = "linear";
		if (mip == "") mip = "nomip";
		if (repeat == "") repeat = "repeat";
		var uv:OGSLAccessor = pop();
		var tex:OGSLAccessor = pop();
		var typeUV:String = env.getVariableType(uv);
		if (env.accessTexture(tex) == null) throw new Error("no such texture: " + tex);
		if (type == "2d") {
			if (typeUV != OGSLConstants.TYPE_VEC2) throw new Error("2D texture coordinates must be vec2");
		} else if (type == "cube") {
			if (typeUV != OGSLConstants.TYPE_VEC3) throw new Error("cube texture coordinates must be vec3");
			repeat = "clamp"; // force clamping
		} else throw new Error("!?");
		tex.flags = type + ", " + filtering + ", " + mip + ", " + repeat;
		var tmp:OGSLAccessor = env.createTempVariable(OGSLConstants.TYPE_VEC4, uv.scope);
		agalop("tex", fld(tmp), fld(uv), fld(tex));
		pusha(tmp);
	}

	private function access():void {
		var accessData:Array = pop().name.split(":");
		var lhs:OGSLAccessor = pop();
		env.combineComponentAccess(lhs, accessData[0], accessData[1]);
		var typeL:String = env.getVariableType(lhs);
		pusha(lhs);
	}

	private function thisAccess():void {
		var rhs:OGSLAccessor = pop();
		rhs.thisAccess = true;
		pusha(rhs);
	}

	private function doReturn(returnTo:OGSLAccessor):void {
		var hasValue:Boolean = pop().name == "1";
		if (!hasValue) {
			if (returnTo != null) throw new Error("function must return data");
			return;
		}
		if (returnTo == null) throw new Error("function must not return data");
		var dataToReturn:OGSLAccessor = pop();
		pusha(returnTo);
		pusha(dataToReturn);
		assign();
	}

	private function assign(doPush:Boolean = true):void {
		var rhs:OGSLAccessor = pop();
		var lhs:OGSLAccessor = pop();
		var typeL:String = env.getVariableType(lhs);
		var typeR:String = env.getVariableType(rhs);
		if (typeL != typeR) {
			var cmp:String;
			if (typeR == OGSLConstants.TYPE_FLOAT) {
				rhs.isAutomaticScalarSwizzlingEnabled = true;
				cmp = rhs.components == "" ? "x" : rhs.components;
				switch(typeL) {
				case OGSLConstants.TYPE_VEC2:
					rhs.components = cmp + cmp;
					break;
				case OGSLConstants.TYPE_VEC3:
					rhs.components = cmp + cmp + cmp;
					break;
				case OGSLConstants.TYPE_VEC4:
				case OGSLConstants.TYPE_MAT3X4:
				case OGSLConstants.TYPE_MAT4X4:
					rhs.components = cmp + cmp + cmp + cmp;
					break;
				default:
					throw new Error("!?");
				}
				typeR = typeL;
			} else throw new Error("types mismatch: " + typeL + " and " + typeR);
		}
		if (!env.isWritable(lhs)) throw new Error("you cannot write to read-only variables");
		checkComponentDuplication(lhs.components);
		typeop2("mov", typeL, lhs, rhs);
		if (doPush) pusha(lhs);
	}

	private function binaryOpEqual(op:String):void {
		var rhs:OGSLAccessor = pop();
		var lhs:OGSLAccessor = pop();
		var typeL:String = env.getVariableType(lhs);
		var typeR:String = env.getVariableType(rhs);
		if (typeL != typeR) {
			var cmp:String;
			if (typeR == OGSLConstants.TYPE_FLOAT) {
				rhs.isAutomaticScalarSwizzlingEnabled = true;
				cmp = rhs.components == "" ? "x" : rhs.components;
				switch(typeL) {
				case OGSLConstants.TYPE_VEC2:
					rhs.components = cmp + cmp;
					break;
				case OGSLConstants.TYPE_VEC3:
					rhs.components = cmp + cmp + cmp;
					break;
				case OGSLConstants.TYPE_VEC4:
				case OGSLConstants.TYPE_MAT3X4:
				case OGSLConstants.TYPE_MAT4X4:
					rhs.components = cmp + cmp + cmp + cmp;
					break;
				default:
					throw new Error("!?");
				}
				typeR = typeL;
			} else throw new Error("types mismatch: " + typeL + " and " + typeR);
		}
		if (!env.isWritable(lhs)) throw new Error("you cannot write to read-only variables");
		checkComponentDuplication(lhs.components);
		typeop3(op, typeL, lhs, lhs, rhs);
		pusha(lhs);
	}

	private function binaryOp(op:String, requiredType:String = "", resultType:String = "", reverse:Boolean = false):void {
		var rhs:OGSLAccessor;
		var lhs:OGSLAccessor;
		if (reverse) {
			lhs = pop();
			rhs = pop();
		} else {
			rhs = pop();
			lhs = pop();
		}
		var typeL:String = env.getVariableType(lhs);
		var typeR:String = env.getVariableType(rhs);
		if (typeL != typeR) {
			var cmp:String;
			if (typeL == OGSLConstants.TYPE_FLOAT) {
				lhs.isAutomaticScalarSwizzlingEnabled = true;
				cmp = lhs.components == "" ? "x" : lhs.components;
				switch(typeR) {
				case OGSLConstants.TYPE_VEC2:
					lhs.components = cmp + cmp;
					break;
				case OGSLConstants.TYPE_VEC3:
					lhs.components = cmp + cmp + cmp;
					break;
				case OGSLConstants.TYPE_VEC4:
				case OGSLConstants.TYPE_MAT3X4:
				case OGSLConstants.TYPE_MAT4X4:
					lhs.components = cmp + cmp + cmp + cmp;
					break;
				default:
					throw new Error("!?");
				}
				typeL = typeR;
			} else if (typeR == OGSLConstants.TYPE_FLOAT) {
				rhs.isAutomaticScalarSwizzlingEnabled = true;
				cmp = rhs.components == "" ? "x" : rhs.components;
				switch(typeL) {
				case OGSLConstants.TYPE_VEC2:
					rhs.components = cmp + cmp;
					break;
				case OGSLConstants.TYPE_VEC3:
					rhs.components = cmp + cmp + cmp;
					break;
				case OGSLConstants.TYPE_VEC4:
				case OGSLConstants.TYPE_MAT3X4:
				case OGSLConstants.TYPE_MAT4X4:
					rhs.components = cmp + cmp + cmp + cmp;
					break;
				default:
					throw new Error("!?");
				}
				typeR = typeL;
			} else throw new Error("types mismatch: " + typeL + " and " + typeR);
		}
		if (requiredType != "" && typeL != requiredType) throw new Error("invalid type: " + typeL);
		// check operation between constants
		if (env.isConstant(lhs) && env.isConstant(rhs)) {
			if (lhs.isLiteralNumberAccess() && rhs.isLiteralNumberAccess()) {
				var ln:Number = parseFloat(lhs.name);
				var rn:Number = parseFloat(rhs.name);
				switch(op) {
				case "add":
					push(String(ln + rn), lhs.scope);
					return;
				case "sub":
					push(String(ln - rn), lhs.scope);
					return;
				case "mul":
					push(String(ln * rn), lhs.scope);
					return;
				case "div":
					push(String(ln / rn), lhs.scope);
					return;
				case "min":
					push(String(ln < rn ? ln : rn), lhs.scope);
					return;
				case "max":
					push(String(ln > rn ? ln : rn), lhs.scope);
					return;
				case "pow":
					push(String(Math.pow(ln, rn)), lhs.scope);
					return;
				}
			}
			pusha(env.createTempVariable(typeL, lhs.scope));
			pusha(lhs);
			assign();
			lhs = pop(); // move lhs to temporary register and use it as lhs
		}
		var tmp:OGSLAccessor = env.createTempVariable(resultType == "" ? typeL : resultType, lhs.scope);
		typeop3(op, typeL, tmp, lhs, rhs);
		pusha(tmp);
	}

	private function unaryOp(op:String):void {
		var rhs:OGSLAccessor = pop();
		var typeR:String = env.getVariableType(rhs);
		// check if rhs is a constant
		if (env.isConstant(rhs)) {
			if (rhs.isLiteralNumberAccess()) {
				var rn:Number = parseFloat(rhs.name);
				switch(op) {
				case "neg":
					push(String(-rn), rhs.scope);
					return;
				case "abs":
					push(String(rn < 0 ? rn : rn), rhs.scope);
					return;
				case "sqt":
					push(String(Math.sqrt(rn)), rhs.scope);
					return;
				case "rcp":
					push(String(1 / rn), rhs.scope);
					return;
				case "sat":
					push(String(rn < 0 ? 0 : rn > 1 ? 1 : rn), rhs.scope);
					return;
				case "sin":
					push(String(Math.sin(rn)), rhs.scope);
					return;
				case "cos":
					push(String(Math.cos(rn)), rhs.scope);
					return;
				}
			}
			pusha(env.createTempVariable(typeR, rhs.scope));
			pusha(rhs);
			assign();
			rhs = pop(); // move rhs to temporary register and use it as rhs
		}
		var tmp:OGSLAccessor = env.createTempVariable(typeR, rhs.scope);
		typeop2(op, typeR, tmp, rhs);
		pusha(tmp);
	}

	private function clamp():void {
		var max:OGSLAccessor = pop();
		var min:OGSLAccessor = pop();
		var a:OGSLAccessor = pop();
		pusha(a);
		pusha(min);
		binaryOp("max");
		pusha(max);
		binaryOp("min");
	}

	private function smoothstep():void { // t * t * (3 - 2 * t) where: t = saturate((a - min) / (max - min));
		var a:OGSLAccessor = pop();
		var max:OGSLAccessor = pop();
		var min:OGSLAccessor = pop();
		var tmp:OGSLAccessor;
		pusha(a);
		pusha(min);
		binaryOp("sub");
		pusha(max);
		pusha(min);
		binaryOp("sub");
		binaryOp("div");
		unaryOp("sat");
		tmp = pop();
		pusha(tmp);
		pusha(tmp);
		binaryOp("mul");
		push("3", a.scope);
		push("2", a.scope);
		pusha(tmp);
		binaryOp("mul");
		binaryOp("sub");
		binaryOp("mul");
	}

	private function mulMat():void {
		var rhs:OGSLAccessor = pop();
		var lhs:OGSLAccessor = pop();
		var typeL:String = env.getVariableType(lhs);
		var typeR:String = env.getVariableType(rhs);
		var tmp:OGSLAccessor;
		var tmpS:String;
		var isVec3:Boolean;
		var isMat3x4:Boolean;
		if ((typeL == OGSLConstants.TYPE_MAT3X4 || typeL == OGSLConstants.TYPE_MAT4X4) && (typeR == OGSLConstants.TYPE_VEC3 || typeR == OGSLConstants.TYPE_VEC4)) {
			tmp = lhs;
			lhs = rhs;
			rhs = tmp;
			tmpS = typeL;
			typeL = typeR;
			typeR = tmpS;
		} else if ((typeL != OGSLConstants.TYPE_VEC3 && typeL != OGSLConstants.TYPE_VEC4) || (typeR != OGSLConstants.TYPE_MAT3X4 && typeR != OGSLConstants.TYPE_MAT4X4)) {
			throw new Error("mul function requires vec3 (or vec4) and mat3x4 (or mat4x4)");
		}
		// check operation between constants
		if (env.isConstant(lhs) && env.isConstant(rhs)) {
			pusha(env.createTempVariable(typeL, lhs.scope));
			pusha(lhs);
			assign();
			lhs = pop(); // move lhs to temporary register and use it as lhs
		}
		isVec3 = typeL == OGSLConstants.TYPE_VEC3;
		isMat3x4 = typeR == OGSLConstants.TYPE_MAT3X4;
		tmp = env.createTempVariable(isVec3 || isMat3x4 ? OGSLConstants.TYPE_VEC3 : OGSLConstants.TYPE_VEC4, lhs.scope);
		agalop(isVec3 ? "m33" : isMat3x4 ? "m34" : "m44", fld(tmp), fld(lhs), fld(rhs));
		pusha(tmp);
	}

	private function distance():void {
		var rhs:OGSLAccessor = pop();
		var lhs:OGSLAccessor = pop();
		var typeL:String = env.getVariableType(lhs);
		var typeR:String = env.getVariableType(rhs);
		var tmp:OGSLAccessor;
		if (typeL != typeR) throw new Error("types mismatch: " + typeL + " and " + typeR);
		if (typeL == OGSLConstants.TYPE_MAT3X4 || typeL == OGSLConstants.TYPE_MAT4X4) throw new Error("unexpected argument type: " + typeL);
		pusha(lhs);
		pusha(rhs);
		binaryOp("sub");
		tmp = pop();
		pusha(tmp);
		pusha(tmp);
		dot();
		unaryOp("sqt");
	}

	private function reflect():void { // vec - 2 * dot(vec, normal) * normal
		var normal:OGSLAccessor = pop();
		var vec:OGSLAccessor = pop();
		pusha(vec);
		pusha(vec);
		pusha(normal);
		dot();
		push("2", vec.scope);
		binaryOp("mul");
		pusha(normal);
		binaryOp("mul");
		binaryOp("sub");
	}

	private function refract():void {
		// k < 0 ? 0 : eta * vec - (eta * dp + sqrt(k)) * normal where: dp = dot(normal, vec), k = 1 - eta * eta * (1 - dp * dp)
		var eta:OGSLAccessor = pop();
		if (env.getVariableType(eta) != OGSLConstants.TYPE_FLOAT) throw new Error("ratio of indices of refraction must be float type");
		var normal:OGSLAccessor = pop();
		var vec:OGSLAccessor = pop();
		pusha(normal);
		pusha(vec);
		dot();
		var dp:OGSLAccessor = pop();
		push("1", vec.scope);
		pusha(eta);
		pusha(eta);
		binaryOp("mul");
		push("1", vec.scope);
		pusha(dp);
		pusha(dp);
		binaryOp("mul");
		binaryOp("sub");
		binaryOp("mul");
		binaryOp("sub");
		var k:OGSLAccessor = pop();
		pusha(eta);
		pusha(vec);
		binaryOp("mul");
		pusha(eta);
		pusha(dp);
		binaryOp("mul");
		pusha(k);
		unaryOp("sqt");
		binaryOp("add");
		pusha(normal);
		binaryOp("mul");
		binaryOp("sub");
		pusha(k);
		push("0", vec.scope);
		binaryOp("sge");
		binaryOp("mul");
	}

	private function mod():void { // fract(a / b) * b
		var b:OGSLAccessor = pop();
		var a:OGSLAccessor = pop();
		pusha(a);
		pusha(b);
		binaryOp("div");
		unaryOp("frc");
		pusha(b);
		binaryOp("mul");
	}

	private function mix():void { // a + (b - a) * t
		var t:OGSLAccessor = pop();
		var b:OGSLAccessor = pop();
		var a:OGSLAccessor = pop();
		pusha(a);
		pusha(b);
		pusha(a);
		binaryOp("sub");
		pusha(t);
		binaryOp("mul");
		binaryOp("add");
	}

	private function floor():void { // a - fract(a)
		var a:OGSLAccessor = pop();
		pusha(a);
		unaryOp("frc");
		var fraction:OGSLAccessor = pop();
		pusha(a);
		pusha(fraction);
		binaryOp("sub");
	}

	private function ceil():void { // a + fract(-a)
		var a:OGSLAccessor = pop();
		pusha(a);
		unaryOp("neg");
		unaryOp("frc");
		var fraction:OGSLAccessor = pop();
		pusha(a);
		pusha(fraction);
		binaryOp("add");
	}

	private function round():void { // floor(a + 0.5)
		var a:OGSLAccessor = pop();
		pusha(a);
		push("0.5", a.scope);
		binaryOp("add");
		floor();
	}

	private function length():void {
		var vec:OGSLAccessor = pop();
		var type:String = env.getVariableType(vec);
		switch(type) {
		case OGSLConstants.TYPE_FLOAT:
			pusha(vec);
			break;
		case OGSLConstants.TYPE_VEC2:
		case OGSLConstants.TYPE_VEC3:
		case OGSLConstants.TYPE_VEC4:
			pusha(vec);
			pusha(vec);
			dot();
			unaryOp("sqt");
			break;
		default:
			throw new Error("unexpected argument type: " + type);
		}
	}

	private function normalize():void {
		var vec:OGSLAccessor = pop();
		var type:String = env.getVariableType(vec);
		var tmp:OGSLAccessor;
		switch(type) {
		case OGSLConstants.TYPE_FLOAT:
			pusha(vec);
			unaryOp("rcp");
			pusha(vec);
			binaryOp("mul");
			break;
		case OGSLConstants.TYPE_VEC2:
		case OGSLConstants.TYPE_VEC4:
			pusha(vec);
			pusha(vec);
			dot();
			unaryOp("rsq");
			pusha(vec);
			binaryOp("mul");
			break;
		case OGSLConstants.TYPE_VEC3:
			pusha(env.createTempVariable(OGSLConstants.TYPE_VEC3, vec.scope));
			pusha(vec);
			unaryOp("nrm");
			assign();
			break;
		default:
			throw new Error("unexpected argument type: " + type);
		}
	}

	private function push(name:String, scope:String = "", thisAccess:Boolean = false, index:String = "", components:String = ""):void {
		stack[stackCount].name = name;
		stack[stackCount].scope = scope;
		stack[stackCount].thisAccess = thisAccess;
		stack[stackCount].index = index;
		stack[stackCount].components = components;
		stackCount++;
	}

	private function checkComponentDuplication(components:String):void {
		var count:int = 0;
		if (components.indexOf("x") != -1) count++;
		if (components.indexOf("y") != -1) count++;
		if (components.indexOf("z") != -1) count++;
		if (components.indexOf("w") != -1) count++;
		if (components.length != count) throw new Error("comopnent duplication is found");
	}

	private function typeop2(op:String, type:String, dst:OGSLAccessor, src:OGSLAccessor):void {
		switch (type) {
		case OGSLConstants.TYPE_FLOAT:
		case OGSLConstants.TYPE_VEC2:
		case OGSLConstants.TYPE_VEC3:
		case OGSLConstants.TYPE_VEC4:
			agalop(op, fld(dst), fld(src));
			break;
		case OGSLConstants.TYPE_MAT3X4:
			if (dst.index != "" || src.index != "") throw new Error("!?");
			dst.index = "0"; src.index = "0";
			agalop(op, fld(dst), fld(src));
			dst.index = "1"; src.index = "1";
			agalop(op, fld(dst), fld(src));
			dst.index = "2"; src.index = "2";
			agalop(op, fld(dst), fld(src));
			dst.index = ""; src.index = "";
			break;
		case OGSLConstants.TYPE_MAT4X4:
			if (dst.index != "" || src.index != "") throw new Error("!?");
			dst.index = "0"; src.index = "0";
			agalop(op, fld(dst), fld(src));
			dst.index = "1"; src.index = "1";
			agalop(op, fld(dst), fld(src));
			dst.index = "2"; src.index = "2";
			agalop(op, fld(dst), fld(src));
			dst.index = "3"; src.index = "3";
			agalop(op, fld(dst), fld(src));
			dst.index = ""; src.index = "";
			break;
		}
	}

	private function typeop3(op:String, type:String, dst:OGSLAccessor, lhs:OGSLAccessor, rhs:OGSLAccessor):void {
		switch (type) {
		case OGSLConstants.TYPE_FLOAT:
		case OGSLConstants.TYPE_VEC2:
		case OGSLConstants.TYPE_VEC3:
		case OGSLConstants.TYPE_VEC4:
			agalop(op, fld(dst), fld(lhs), fld(rhs));
			break;
		case OGSLConstants.TYPE_MAT3X4:
			if (dst.index != "" || lhs.index != "" || rhs.index != "") throw new Error("!?");
			dst.index = "0"; lhs.index = "0"; rhs.index = "0";
			agalop(op, fld(dst), fld(lhs), fld(rhs));
			dst.index = "1"; lhs.index = "1"; rhs.index = "1";
			agalop(op, fld(dst), fld(lhs), fld(rhs));
			dst.index = "2"; lhs.index = "2"; rhs.index = "2";
			agalop(op, fld(dst), fld(lhs), fld(rhs));
			dst.index = ""; lhs.index = ""; rhs.index = "";
			break;
		case OGSLConstants.TYPE_MAT4X4:
			if (dst.index != "" || lhs.index != "" || rhs.index != "") throw new Error("!?");
			dst.index = "0"; lhs.index = "0"; rhs.index = "0";
			agalop(op, fld(dst), fld(lhs), fld(rhs));
			dst.index = "1"; lhs.index = "1"; rhs.index = "1";
			agalop(op, fld(dst), fld(lhs), fld(rhs));
			dst.index = "2"; lhs.index = "2"; rhs.index = "2";
			agalop(op, fld(dst), fld(lhs), fld(rhs));
			dst.index = "3"; lhs.index = "3"; rhs.index = "3";
			agalop(op, fld(dst), fld(lhs), fld(rhs));
			dst.index = ""; lhs.index = ""; rhs.index = "";
			break;
		}
	}

	private function agalop(op:String, dst:OGSLField = null, src1:OGSLField = null, src2:OGSLField = null):void {
		append(new OGSLLine(op, dst, src1, src2));
	}

	private function fld(accessor:OGSLAccessor):OGSLField {
		return env.getField(accessor);
	}

	private function pusha(accessor:OGSLAccessor):void {
		push(accessor.name, accessor.scope, accessor.thisAccess, accessor.index, accessor.components);
	}

	private function pop():OGSLAccessor {
		return stack[--stackCount].clone();
	}

	private function append(line:OGSLLine):void {
		if (emitMode == 0) vertexOutput.append(line);
		else if (emitMode == 1) fragmentOutput.append(line);
		else throw new Error("!?");
	}

}

class OGSLOptimizer {
	private var log:Function;
	private var output:OGSLOutput;
	private var isVertex:Boolean;
	private var tempVariableCount:int;
	private var tempRegisterName:String;
	private var allocator:OGSLRegisterIndexAllocator;

	public function OGSLOptimizer(logFunction:Function) {
		log = logFunction;
	}

	public function optimize(output:OGSLOutput, type:String, maxPasses:int = 64):void {
		this.output = output;
		switch (type) {
		case OGSLConstants.PROGRAM_TYPE_VERTEX:
			isVertex = true;
			tempRegisterName = "vt";
			break;
		case OGSLConstants.PROGRAM_TYPE_FRAGMENT:
			isVertex = false;
			tempRegisterName = "ft";
			break;
		default:
			throw new Error("!?");
		}
		allocator = new OGSLRegisterIndexAllocator(isVertex ? OGSLConstants.MAX_VERTEX_TEMPORARY_REGISTERS : OGSLConstants.MAX_FRAGMENT_TEMPORARY_REGISTERS);
		var lastCode:String = output.print();
		for (var i:int = 0; i < maxPasses; i++) {
			shrinkLines();
			optimizeLines();
			shrinkTempVariables();
			var newCode:String = output.print();
			if (lastCode == newCode) {
				log(type + " program optimization finished in " + (i + 1) + " pass(es)");
				break;
			}
			lastCode = newCode;
		}
		correctSwizzling();
		complementVaryingOutput();
	}

	private function correctSwizzling():void {
		var line:OGSLLine = output.lines;
		while (line) {
			if (line.dst && line.dst.components.length > 1) {
				var cmp:String = line.dst.components;
				var mask:int = line.dst.getComponentMask();
				if (cmp != "xy" && cmp != "xyz" && cmp != "xyzw") { // dst needs correcting swizzling
					var sc1:String = line.src1.components;
					var sc2:String = line.src2 ? line.src2.components : cmp;
					if (sc1 == "") sc1 = "xyzw";
					if (sc2 == "") sc2 = "xyzw";
					// complement
					if ((mask & 8) == 0) {
						cmp += "x";
						sc1 += sc1.charAt(0);
						sc2 += sc2.charAt(0);
					}
					if ((mask & 4) == 0) {
						cmp += "y";
						sc1 += sc1.charAt(0);
						sc2 += sc2.charAt(0);
					}
					if ((mask & 2) == 0) {
						cmp += "z";
						sc1 += sc1.charAt(0);
						sc2 += sc2.charAt(0);
					}
					if ((mask & 1) == 0) {
						cmp += "w";
						sc1 += sc1.charAt(0);
						sc2 += sc2.charAt(0);
					}
					if (cmp.length != 4 || sc1.length != 4 || sc2.length != 4) throw new Error("!?");
					// sort
					var idxX:int = cmp.indexOf("x");
					var idxY:int = cmp.indexOf("y");
					var idxZ:int = cmp.indexOf("z");
					var idxW:int = cmp.indexOf("w");
					cmp = "xyzw";
					sc1 = sc1.charAt(idxX) + sc1.charAt(idxY) + sc1.charAt(idxZ) + sc1.charAt(idxW);
					sc2 = sc2.charAt(idxX) + sc2.charAt(idxY) + sc2.charAt(idxZ) + sc2.charAt(idxW);
					// shrink
					if ((mask & 8) == 0) cmp = cmp.replace("x", "");
					if ((mask & 4) == 0) cmp = cmp.replace("y", "");
					if ((mask & 2) == 0) cmp = cmp.replace("z", "");
					if ((mask & 1) == 0) cmp = cmp.replace("w", "");
					// set
					line.dst.components = cmp;
					line.src1.components = sc1;
					if (line.src2) line.src2.components = sc2;
				}
			}
			line = line.next;
		}
	}

	private function shrinkLines():void {
		var line:OGSLLine = output.lines;
		while (line) {
			line.toBeRemoved = line.dst && line.dst.isTemporary() && isWritingUseless(line);
			line = line.next;
		}
		line = output.lines;
		while (line) {
			if (line.toBeRemoved) removeLine(line);
			line = line.next;
		}
		line = output.lines;
		while (line) {
			if (line.op == "mov" && line.src1.equals(line.dst)) {
				removeLine(line);
			} else if (line.isConditionalOperation() && line.op != "eif") {
				if (line.op == "els") {
					if (line.next.op == "eif") {
						// ------ before ------
						// els;
						// eif;
						// ...
						// ------ after ------
						// eif;
						// ...
						removeLine(line);
					}
				} else {
					if (line.next.op == "eif") {
						// ------ before ------
						// if;
						// eif;
						// ...
						// ------ after ------
						// ...
						removeLine(line.next);
						removeLine(line);
					} else if (line.next.op == "els") {
						// ------ before ------
						// if;
						// els;
						// ...
						// ------ after ------
						// !if;
						// ...
						line.reverseCondition();
						removeLine(line.next);
					}
				}
			}
			line = line.next;
		}
	}

	private function isWritingUseless(startLine:OGSLLine):Boolean {
		if (!startLine.dst || !startLine.dst.isTemporary()) throw new Error("!?");
		var line:OGSLLine = startLine;
		var name:String = line.dst.name;
		var mask:int = line.dst.getComponentMask();
		while (line = line.next) {
			if (line.hasOverlappedAccessToSrc1(name, mask)) return false;
			if (line.hasOverlappedAccessToSrc2(name, mask)) return false;
			if (line.hasOverlappedAccessToDst(name, mask) && willLineCertainlyBeRun(startLine, line)) {
				mask &= ~line.dst.getComponentMask();
				if (mask == 0) return true;
			}
		}
		return true;
	}

	private function shrinkTempVariables():void {
		var regName:String = isVertex ? "vt" : "ft";
		// init the data
		var line:OGSLLine = output.lines;
		var lineCount:int = 0;
		while (line) {
			line.index = lineCount++;
			line = line.next;
		}
		// calculate variable ranges
		line = output.lines;
		tempVariableCount = 0;
		var blockId:int = 1;
		var maxTempRegisters:int = isVertex ? OGSLConstants.MAX_VERTEX_TEMPORARY_REGISTERS : OGSLConstants.MAX_FRAGMENT_TEMPORARY_REGISTERS;
		var mapWidth:int = maxTempRegisters * 4;
		var mapHeight:int = lineCount * 2;
		// map format:
		//   LLLLLLLL LLLLLLLL IIIIIIII IIIIIIII
		// where: L = the amount of movements of the register to the left(lower register index)
		//        I = block id
		var tempRegisterMap:Vector.<Vector.<int>> = new Vector.<Vector.<int>>(mapHeight, true);
		for (var i:int = 0; i < mapHeight; i++) {
			tempRegisterMap[i] = new Vector.<int>(mapWidth, true);
		}
		var blockAABB:Vector.<int> = new Vector.<int>(lineCount * 4, true);
		for (i = 0; i < lineCount; i++) {
			blockAABB[(i << 2)] = mapWidth;
			blockAABB[(i << 2) + 1] = -1;
			blockAABB[(i << 2) + 2] = mapHeight;
			blockAABB[(i << 2) + 3] = -1;
		}
		while (line) {
			if (line.dst && line.dst.isTemporary()) {
				if (fillBlockId(tempRegisterMap, blockAABB, line, blockId)) blockId++;
			}
			line = line.next;
		}

		combineBlocks(tempRegisterMap, blockAABB);

		for (i = 1; i < blockId; i++) {
			moveBlockLeft(tempRegisterMap, blockAABB, mapWidth, mapHeight, i);
		}

		line = output.lines;
		var mapIndex:int = 0;
		while (line) {
			var amountOfMovement:int;
			var mask:int;
			var regIndexOffset:int;
			var f:OGSLField;
			if (line.src1 && line.src1.isTemporary()) {
				f = line.src1;
				mask = f.getComponentMask();
				amountOfMovement = -1;
				regIndexOffset = f.getIndex() << 2;
				if ((mask & 8) != 0) amountOfMovement = tempRegisterMap[mapIndex][regIndexOffset] >>> 16;
				regIndexOffset++;
				if ((mask & 4) != 0) amountOfMovement = tempRegisterMap[mapIndex][regIndexOffset] >>> 16;
				regIndexOffset++;
				if ((mask & 2) != 0) amountOfMovement = tempRegisterMap[mapIndex][regIndexOffset] >>> 16;
				regIndexOffset++;
				if ((mask & 1) != 0) amountOfMovement = tempRegisterMap[mapIndex][regIndexOffset] >>> 16;
				if (amountOfMovement == -1) throw new Error("!?");
				if (amountOfMovement > 0) f.moveComponentsLeft(amountOfMovement);
			}
			if (line.src2 && line.src2.isTemporary()) {
				f = line.src2;
				mask = f.getComponentMask();
				amountOfMovement = -1;
				regIndexOffset = f.getIndex() << 2;
				if ((mask & 8) != 0) amountOfMovement = tempRegisterMap[mapIndex][regIndexOffset] >>> 16;
				regIndexOffset++;
				if ((mask & 4) != 0) amountOfMovement = tempRegisterMap[mapIndex][regIndexOffset] >>> 16;
				regIndexOffset++;
				if ((mask & 2) != 0) amountOfMovement = tempRegisterMap[mapIndex][regIndexOffset] >>> 16;
				regIndexOffset++;
				if ((mask & 1) != 0)amountOfMovement = tempRegisterMap[mapIndex][regIndexOffset] >>> 16;
				if (amountOfMovement == -1) throw new Error("!?");
				if (amountOfMovement > 0) f.moveComponentsLeft(amountOfMovement);
			}
			mapIndex++;
			if (line.dst && line.dst.isTemporary()) {
				f = line.dst;
				mask = f.getComponentMask();
				amountOfMovement = -1;
				regIndexOffset = f.getIndex() << 2;
				if ((mask & 8) != 0) amountOfMovement = tempRegisterMap[mapIndex][regIndexOffset] >>> 16;
				regIndexOffset++;
				if ((mask & 4) != 0) amountOfMovement = tempRegisterMap[mapIndex][regIndexOffset] >>> 16;
				regIndexOffset++;
				if ((mask & 2) != 0) amountOfMovement = tempRegisterMap[mapIndex][regIndexOffset] >>> 16;
				regIndexOffset++;
				if ((mask & 1) != 0)amountOfMovement = tempRegisterMap[mapIndex][regIndexOffset] >>> 16;
				if (amountOfMovement == -1) throw new Error("!?");
				if (amountOfMovement > 0) f.moveComponentsLeft(amountOfMovement);
			}
			mapIndex++;
			line = line.next;
		}
	}

	private function combineBlocks(map:Vector.<Vector.<int>>, blockAABB:Vector.<int>):void {
		var mapIndex:int = 0;
		var line:OGSLLine = output.lines;
		while (line) {
			var mask:int;
			var regIndexOffset:int;
			var f:OGSLField;
			var x:Boolean;
			var y:Boolean;
			var z:Boolean;
			var w:Boolean;
			f = line.src1;
			if (f && f.isTemporary()) {
				mask = f.getComponentMask();
				regIndexOffset = f.getIndex() << 2;
				x = (mask & 8) != 0;
				y = (mask & 4) != 0;
				z = (mask & 2) != 0;
				w = (mask & 1) != 0;
				if (x) { // accesses to component x first
					if (y) combineBlocksById(map, blockAABB, map[mapIndex][regIndexOffset], map[mapIndex][regIndexOffset + 1]);
					if (z) combineBlocksById(map, blockAABB, map[mapIndex][regIndexOffset], map[mapIndex][regIndexOffset + 2]);
					if (w) combineBlocksById(map, blockAABB, map[mapIndex][regIndexOffset], map[mapIndex][regIndexOffset + 3]);
				} else if (y) { // accesses to component y first
					if (z) combineBlocksById(map, blockAABB, map[mapIndex][regIndexOffset + 1], map[mapIndex][regIndexOffset + 2]);
					if (w) combineBlocksById(map, blockAABB, map[mapIndex][regIndexOffset + 1], map[mapIndex][regIndexOffset + 3]);
				} else if (z) { // accesses to component z first
					if (w) combineBlocksById(map, blockAABB, map[mapIndex][regIndexOffset + 2], map[mapIndex][regIndexOffset + 3]);
				}
			}
			f = line.src2;
			if (f && f.isTemporary()) {
				if (line.op == "m33" || line.op == "m34" || line.op == "m44") { // special matrix access
					// mat3x4 or mat4x4 access only: there's no mat3x3 access, see
					//     http://helpx.adobe.com/flash-player/kb/agal-command-m33-requires-3x4.html
					var numComponents:int = line.op == "m44" ? 16 : 12;
					regIndexOffset = f.getIndex() << 2;
					for (var i:int = 0; i < numComponents; i++) {
						combineBlocksById(map, blockAABB, map[mapIndex][regIndexOffset], map[mapIndex][regIndexOffset + i]);
					}
				} else { // normal access
					mask = f.getComponentMask();
					regIndexOffset = f.getIndex() << 2;
					x = (mask & 8) != 0;
					y = (mask & 4) != 0;
					z = (mask & 2) != 0;
					w = (mask & 1) != 0;
					if (x) { // accesses to component x first
						if (y) combineBlocksById(map, blockAABB, map[mapIndex][regIndexOffset], map[mapIndex][regIndexOffset + 1]);
						if (z) combineBlocksById(map, blockAABB, map[mapIndex][regIndexOffset], map[mapIndex][regIndexOffset + 2]);
						if (w) combineBlocksById(map, blockAABB, map[mapIndex][regIndexOffset], map[mapIndex][regIndexOffset + 3]);
					} else if (y) { // accesses to component y first
						if (z) combineBlocksById(map, blockAABB, map[mapIndex][regIndexOffset + 1], map[mapIndex][regIndexOffset + 2]);
						if (w) combineBlocksById(map, blockAABB, map[mapIndex][regIndexOffset + 1], map[mapIndex][regIndexOffset + 3]);
					} else if (z) { // accesses to component z first
						if (w) combineBlocksById(map, blockAABB, map[mapIndex][regIndexOffset + 2], map[mapIndex][regIndexOffset + 3]);
					}
				}
			}
			mapIndex++;
			f = line.dst;
			if (f && f.isTemporary()) {
				mask = f.getComponentMask();
				regIndexOffset = f.getIndex() << 2;
				x = (mask & 8) != 0;
				y = (mask & 4) != 0;
				z = (mask & 2) != 0;
				w = (mask & 1) != 0;
				if (x) { // accesses to component x first
					if (y) combineBlocksById(map, blockAABB, map[mapIndex][regIndexOffset], map[mapIndex][regIndexOffset + 1]);
					if (z) combineBlocksById(map, blockAABB, map[mapIndex][regIndexOffset], map[mapIndex][regIndexOffset + 2]);
					if (w) combineBlocksById(map, blockAABB, map[mapIndex][regIndexOffset], map[mapIndex][regIndexOffset + 3]);
				} else if (y) { // accesses to component y first
					if (z) combineBlocksById(map, blockAABB, map[mapIndex][regIndexOffset + 1], map[mapIndex][regIndexOffset + 2]);
					if (w) combineBlocksById(map, blockAABB, map[mapIndex][regIndexOffset + 1], map[mapIndex][regIndexOffset + 3]);
				} else if (z) { // accesses to component z first
					if (w) combineBlocksById(map, blockAABB, map[mapIndex][regIndexOffset + 2], map[mapIndex][regIndexOffset + 3]);
				}
			}
			mapIndex++;
			line = line.next;
		}
	}

	private function combineBlocksById(map:Vector.<Vector.<int>>, blockAABB:Vector.<int>, idA:int, idB:int):void {
		if (idA == idB) return;
		if (idA > idB) { // swap
			idA ^= idB;
			idB ^= idA;
			idA ^= idB;
		}
		var aabbOffsetA:int = idA - 1 << 2;
		var aabbOffsetB:int = idB - 1 << 2;
		var x1A:int = blockAABB[aabbOffsetA + 0];
		var x2A:int = blockAABB[aabbOffsetA + 1];
		var y1A:int = blockAABB[aabbOffsetA + 2];
		var y2A:int = blockAABB[aabbOffsetA + 3];
		var x1B:int = blockAABB[aabbOffsetB + 0];
		var x2B:int = blockAABB[aabbOffsetB + 1];
		var y1B:int = blockAABB[aabbOffsetB + 2];
		var y2B:int = blockAABB[aabbOffsetB + 3];
		// combine IDs
		for (var i:int = y1B; i <= y2B; i++) {
			for (var j:int = x1B; j <= x2B; j++) {
				if (map[i][j] == idB) map[i][j] = idA;
			}
		}
		// combine AABBs
		if (x1B < x1A) x1A = x1B;
		if (x2B > x2A) x2A = x2B;
		if (y1B < y1A) y1A = y1B;
		if (y2B > y2A) y2A = y2B;
		blockAABB[aabbOffsetA + 0] = x1A;
		blockAABB[aabbOffsetA + 1] = x2A;
		blockAABB[aabbOffsetA + 2] = y1A;
		blockAABB[aabbOffsetA + 3] = y2A;
		blockAABB[aabbOffsetB] = -1; // delete AABB
	}

	private function moveBlockLeft(map:Vector.<Vector.<int>>, blockAABB:Vector.<int>, w:int, h:int, blockId:int):void {
		// calc accurate aabb of the block
		var offset:int = blockId - 1 << 2;
		var x1:int = blockAABB[offset];
		var x2:int = blockAABB[offset + 1];
		var y1:int = blockAABB[offset + 2];
		var y2:int = blockAABB[offset + 3];
		if (x1 == -1) return; // no block
		// how much the block can be moved to left?
		var tryingAmount:int = x1 + 1;
		var fourUnit:Boolean = x2 - x1 != 0; // be careful of "crs" or "nrm"...
		OUTER: while (--tryingAmount > 0) {
			if (fourUnit && (tryingAmount & 3) != 0) continue; // nope
			for (var i:int = y1; i <= y2; i++) {
				for (var j:int = x1; j <= x2; j++) {
					if (map[i][j] == blockId && ((map[i][j - tryingAmount] & 0xffff) != 0)) {
						continue OUTER;
					}
				}
			}
			break;
		}
		if (tryingAmount == 0) return;
		// move the block
		for (i = y1; i <= y2; i++) {
			for (j = x1; j <= x2; j++) {
				if (map[i][j] == blockId) {
					map[i][j - tryingAmount] |= blockId;
					map[i][j] = tryingAmount << 16;
				}
			}
		}
	}

	private function fillBlockId(map:Vector.<Vector.<int>>, blockAABB:Vector.<int>, startLine:OGSLLine, blockId:int):Boolean {
		var mapIndex:int = startLine.index * 2 + 1; // + 0: src, + 1: dst
		var mask:int = startLine.dst.getComponentMask();
		var regIndex:int = startLine.dst.getIndex() << 2;
		var idUsed:Boolean = false;
		if ((mask & 8) != 0 && map[mapIndex][regIndex] == 0) { // the line writes to dst.x
			fillBlockIdComponent(map, blockAABB, startLine, mapIndex, regIndex, blockId);
			idUsed = true;
		}
		if ((mask & 4) != 0 && map[mapIndex][regIndex + 1] == 0) { // the line writes to dst.y
			fillBlockIdComponent(map, blockAABB, startLine, mapIndex, regIndex + 1, blockId);
			idUsed = true;
		}
		if ((mask & 2) != 0 && map[mapIndex][regIndex + 2] == 0) { // the line writes to dst.z
			fillBlockIdComponent(map, blockAABB, startLine, mapIndex, regIndex + 2, blockId);
			idUsed = true;
		}
		if ((mask & 1) != 0 && map[mapIndex][regIndex + 3] == 0) { // the line writes to dst.w
			fillBlockIdComponent(map, blockAABB, startLine, mapIndex, regIndex + 3, blockId);
			idUsed = true;
		}
		return idUsed;
	}

	private function fillBlockIdComponent(map:Vector.<Vector.<int>>, blockAABB:Vector.<int>, startLine:OGSLLine, mapIndex:int, regIndex:int, blockId:int):void {
		if (!startLine) return;
		if (mapIndex >> 1 != startLine.index) throw new Error("!?");
		if (map[mapIndex][regIndex] == blockId) return; // already filled

		var line:OGSLLine = startLine;
		var regName:String = tempRegisterName + (regIndex >> 2);
		var mask:int = 1 << 3 - (regIndex & 3);
		var mapIndexStart:int = mapIndex;
		var mapIndexEnd:int = mapIndex;

		// skip first line
		map[mapIndex][regIndex] = blockId;
		if ((mapIndex & 1) != 0) line = line.next;
		mapIndex++;

		while (mapIndex >= 0 && mapIndex < map.length) {
			if ((mapIndex & 1) == 0) { // src
				map[mapIndex][regIndex] = blockId;
				if (line.hasOverlappedAccessToSrc1(regName, mask)) mapIndexEnd = mapIndex;
				if (line.hasOverlappedAccessToSrc2(regName, mask)) mapIndexEnd = mapIndex;
			} else { // dst
				if (line.hasOverlappedAccessToDst(regName, mask) && willLineCertainlyBeRun(startLine, line)) break;
				map[mapIndex][regIndex] = blockId;
				line = line.next; // go to next line
			}
			mapIndex++;
		}
		while (--mapIndex > mapIndexEnd) { // shrink the end of the group until last access
			map[mapIndex][regIndex] = 0;
		}

		// expand AABB
		var aabbOffset:int = blockId - 1 << 2;
		var x1:int = regIndex;
		var x2:int = regIndex;
		var y1:int = mapIndexStart;
		var y2:int = mapIndexEnd;
		if (blockAABB[aabbOffset + 0] > x1) blockAABB[aabbOffset + 0] = x1;
		if (blockAABB[aabbOffset + 1] < x2) blockAABB[aabbOffset + 1] = x2;
		if (blockAABB[aabbOffset + 2] > y1) blockAABB[aabbOffset + 2] = y1;
		if (blockAABB[aabbOffset + 3] < y2) blockAABB[aabbOffset + 3] = y2;
	}

	private function willLineCertainlyBeRun(from:OGSLLine, target:OGSLLine):Boolean {
		var stackDepth:int = 0;
		var minStackDepth:int = 0;
		var line:OGSLLine = from;
		while (line != target) {
			if (line.op == "ife" || line.op == "ine" || line.op == "ifg" || line.op == "ifl") { // "if"
				stackDepth++;
			} else if (line.op == "els") { // "else" = "endif" + "if"
				if (minStackDepth == stackDepth) minStackDepth--;
			} else if (line.op == "eif") { // "endif"
				stackDepth--;
			}
			if (stackDepth < minStackDepth) minStackDepth = stackDepth;
			line = line.next;
		}
		return minStackDepth == stackDepth;
	}

	private function fillBlockIdComponent_old(map:Vector.<Vector.<int>>, blockAABB:Vector.<int>, startLine:OGSLLine, mapIndex:int, regIndex:int, blockId:int, reverse:Boolean = false):void {
		if (!startLine) return;
		if (mapIndex >> 1 != startLine.index) throw new Error("!?");
		if (map[mapIndex][regIndex] == blockId) return; // already filled

		var regIndexOffset:int = regIndex >> 2 << 2; // points to component x
		var cmpIndex:int = regIndex & 3;
		var line:OGSLLine = startLine;
		var regName:String = tempRegisterName + (regIndex >> 2);
		var mask:int = 1 << 3 - cmpIndex;
		var begunFromDst:Boolean = (mapIndex & 1) != 0;
		var lastAccessedIndex:int = mapIndex;
		var eraseMode:Boolean = reverse && begunFromDst;

		// skip first line
		if (!eraseMode) {
			map[mapIndex][regIndex] = blockId;
			expandAABB(blockAABB, regIndex, mapIndex, blockId);
		}
		if (reverse) {
			if ((mapIndex & 1) == 0) line = line.prev;
			mapIndex--;
		} else {
			if ((mapIndex & 1) != 0) line = line.next;
			mapIndex++;
		}

		while (mapIndex >= 0 && mapIndex < map.length) {
			var accessMask:int;
			var accessOffset:int;
			var accessIndex:int;
			if ((mapIndex & 1) == 0) { // src
				// if (map[mapIndex][regIndex] != 0) throw new Error("!?");
				map[mapIndex][regIndex] = blockId;
				expandAABB(blockAABB, regIndex, mapIndex, blockId);
				if (line.hasOverlappedAccessToSrc1(regName, mask)) {
					lastAccessedIndex = mapIndex;
					eraseMode = false;
					accessMask = line.src1.getComponentMask();
					if (cmpIndex != 0 && (accessMask & 8) != 0) { // the line accesses to component x
						fillBlockIdComponent_old(map, blockAABB, line, mapIndex, regIndexOffset, blockId, true);
						fillBlockIdComponent_old(map, blockAABB, line, mapIndex, regIndexOffset, blockId);
					}
					if (cmpIndex != 1 && (accessMask & 4) != 0) { // the line accesses to component y
						fillBlockIdComponent_old(map, blockAABB, line, mapIndex, regIndexOffset + 1, blockId, true);
						fillBlockIdComponent_old(map, blockAABB, line, mapIndex, regIndexOffset + 1, blockId);
					}
					if (cmpIndex != 2 && (accessMask & 2) != 0) { // the line accesses to component z
						fillBlockIdComponent_old(map, blockAABB, line, mapIndex, regIndexOffset + 2, blockId, true);
						fillBlockIdComponent_old(map, blockAABB, line, mapIndex, regIndexOffset + 2, blockId);
					}
					if (cmpIndex != 3 && (accessMask & 1) != 0) { // the line accesses to component w
						fillBlockIdComponent_old(map, blockAABB, line, mapIndex, regIndexOffset + 3, blockId, true);
						fillBlockIdComponent_old(map, blockAABB, line, mapIndex, regIndexOffset + 3, blockId);
					}
				}
				if (line.hasOverlappedAccessToSrc2(regName, mask)) {
					lastAccessedIndex = mapIndex;
					eraseMode = false;
					if (line.op == "m33" || line.op == "m34" || line.op == "m44") { // special matrix access
						// there's no mat3x3 access: see helpx.adobe.com/flash-player/kb/agal-command-m33-requires-3x4.html
						var numAccesses:int = line.op == "m44" ? 16 : 12;
						accessOffset = line.src2.getIndex() * 4;
						for (var i:int = 0; i < numAccesses; i++) {
							accessIndex = accessOffset + i;
							if (accessIndex != regIndex) {
								fillBlockIdComponent_old(map, blockAABB, line, mapIndex, accessIndex, blockId, true);
								fillBlockIdComponent_old(map, blockAABB, line, mapIndex, accessIndex, blockId);
							}
						}
					} else { // normal access
						accessMask = line.src2.getComponentMask();
						if (cmpIndex != 0 && (accessMask & 8) != 0) { // the line accesses to component x
							fillBlockIdComponent_old(map, blockAABB, line, mapIndex, regIndexOffset, blockId, true);
							fillBlockIdComponent_old(map, blockAABB, line, mapIndex, regIndexOffset, blockId);
						}
						if (cmpIndex != 1 && (accessMask & 4) != 0) { // the line accesses to component y
							fillBlockIdComponent_old(map, blockAABB, line, mapIndex, regIndexOffset + 1, blockId, true);
							fillBlockIdComponent_old(map, blockAABB, line, mapIndex, regIndexOffset + 1, blockId);
						}
						if (cmpIndex != 2 && (accessMask & 2) != 0) { // the line accesses to component z
							fillBlockIdComponent_old(map, blockAABB, line, mapIndex, regIndexOffset + 2, blockId, true);
							fillBlockIdComponent_old(map, blockAABB, line, mapIndex, regIndexOffset + 2, blockId);
						}
						if (cmpIndex != 3 && (accessMask & 1) != 0) { // the line accesses to component w
							fillBlockIdComponent_old(map, blockAABB, line, mapIndex, regIndexOffset + 3, blockId, true);
							fillBlockIdComponent_old(map, blockAABB, line, mapIndex, regIndexOffset + 3, blockId);
						}
					}
				}
				if (reverse) line = line.prev; // go to previous line
			} else { // dst
				if (line.hasOverlappedAccessToDst(regName, mask)) {
					accessMask = line.dst.getComponentMask();
					if (reverse) {
						// definition point found
						map[mapIndex][regIndex] = blockId;
						expandAABB(blockAABB, regIndex, mapIndex, blockId);
						if (cmpIndex != 0 && (accessMask & 8) != 0) { // the line writes to component x
							fillBlockIdComponent_old(map, blockAABB, line, mapIndex, regIndexOffset, blockId);
						}
						if (cmpIndex != 1 && (accessMask & 4) != 0) { // the line writes to component y
							fillBlockIdComponent_old(map, blockAABB, line, mapIndex, regIndexOffset + 1, blockId);
						}
						if (cmpIndex != 2 && (accessMask & 2) != 0) { // the line writes to component z
							fillBlockIdComponent_old(map, blockAABB, line, mapIndex, regIndexOffset + 2, blockId);
						}
						if (cmpIndex != 3 && (accessMask & 1) != 0) { // the line writes to component w
							fillBlockIdComponent_old(map, blockAABB, line, mapIndex, regIndexOffset + 3, blockId);
						}
					} else {
						// the component scope is end: do not write the group id (set 7th argument false)
						if (cmpIndex != 0 && (accessMask & 8) != 0) { // the line writes to component x
							fillBlockIdComponent_old(map, blockAABB, line, mapIndex, regIndexOffset, blockId, true);
						}
						if (cmpIndex != 1 && (accessMask & 4) != 0) { // the line writes to component y
							fillBlockIdComponent_old(map, blockAABB, line, mapIndex, regIndexOffset + 1, blockId, true);
						}
						if (cmpIndex != 2 && (accessMask & 2) != 0) { // the line writes to component z
							fillBlockIdComponent_old(map, blockAABB, line, mapIndex, regIndexOffset + 2, blockId, true);
						}
						if (cmpIndex != 3 && (accessMask & 1) != 0) { // the line writes to component w
							fillBlockIdComponent_old(map, blockAABB, line, mapIndex, regIndexOffset + 3, blockId, true);
						}
					}
					break;
				} else {
					// if (map[mapIndex][regIndex] != 0) throw new Error("!?");
					map[mapIndex][regIndex] = blockId;
					expandAABB(blockAABB, regIndex, mapIndex, blockId);
				}
				if (!reverse) line = line.next; // go to next line
			}
			if (reverse) {
				if (eraseMode) map[mapIndex][regIndex] = 0;
				mapIndex--;
			} else mapIndex++;
		}
		if (!reverse) {
			while (--mapIndex > lastAccessedIndex) { // shrink the end of the group until last access
				map[mapIndex][regIndex] = 0;
			}
		}
	}

	private function expandAABB(blockAABB:Vector.<int>, x:int, y:int, blockId:int):void {
		var offset:int = blockId - 1 << 2;
		var x1:int = blockAABB[offset];
		var x2:int = blockAABB[offset + 1];
		var y1:int = blockAABB[offset + 2];
		var y2:int = blockAABB[offset + 3];
		if (x < x1) x1 = x;
		if (x > x2) x2 = x;
		if (y < y1) y1 = y;
		if (y > y2) y2 = y;
		blockAABB[offset] = x1;
		blockAABB[offset + 1] = x2;
		blockAABB[offset + 2] = y1;
		blockAABB[offset + 3] = y2;
	}

	private function optimizeLines():void {
		var line:OGSLLine = output.lines;
		var r:OGSLLine;
		while (line) {
			OUTER: do {
				if (line.dst) {
					r = line;
					while ((r = r.next) != null && !r.isConditionalOperation()) {
						if (r.op == "mov" && line.dst.equals(r.src1) && !willFieldBeReferedAfter(r, line.dst, true, false)) {
							if (r.prev == line || noAccessBetween(line.next, r.prev, r.dst) && noAccessBetween(line.next, r.prev, r.src1)) {
								// ---- before ----
								// foo v0 v1 v2;
								// ... (does not access to v0 and v3)
								// mov v3 v0;
								// ... (does not read v0)
								// ---- after ----
								// foo v3 v1 v2;
								// ...
								line.dst = r.dst;
								// remove related line from the list
								removeLine(r);
								break OUTER;
							}
						}
					}
				}
				if (line.op == "mov") {
					r = line;
					while ((r = r.next) != null && !r.isConditionalOperation()) {
						if (r.src1 && line.dst.equals(r.src1) && !willFieldBeReferedAfter(r, line.dst, true, false)) {
							if (r.prev == line || noAccessBetween(line.next, r.prev, line.dst) && noAccessBetween(line.next, r.prev, line.src1, true)) {
								// ---- before ----
								// mov v0 v1;
								// ... (does not access to v0 and write to v1)
								// foo v2 v0 v3;
								// ... (does not read v0)
								// ---- after ----
								// ...
								// foo v2 v1 v3;
								// ...
								if (!line.src1.isConstant() || r.src2 && !r.src2.isConstant()) { // avoid making operation between constant registers (causes an error)
									r.src1 = line.src1;
									// remove this line from the list
									removeLine(line);
									break OUTER;
								}
							}
						}
						if (r.op != "m33" && r.op != "m34" && r.op != "m44" && r.src2 && line.dst.equals(r.src2) && !willFieldBeReferedAfter(r, line.dst, false, true)) {
							if (r.prev == line || noAccessBetween(line.next, r.prev, line.dst) && noAccessBetween(line.next, r.prev, line.src1, true)) {
								// ---- before ----
								// mov v0 v1;
								// ... (does not access to v0 and write to v1)
								// foo v2 v3 v0;
								// ... (does not read v0)
								// ---- after ----
								// ...
								// foo v2 v3 v1;
								// ...
								if (!line.src1.isConstant() || !r.src1.isConstant()) { // avoid making operation between constant registers (causes an error)
									r.src2 = line.src1;
									// remove this line from the list
									removeLine(line);
									break OUTER;
								}
							}
						}
					}
				}
				if (line.dst) {
					r = line;
					var hasSrc2:Boolean = line.src2 != null;
					while ((r = r.next) != null && !r.isConditionalOperation() && r.op != "tex") {
						if (r.op == line.op && line.dst.name == r.dst.name && line.src1.name == r.src1.name && (!hasSrc2 || line.src2.name == r.src2.name) && (line.dst.getComponentMask() & r.dst.getComponentMask()) == 0) {
							if (r.prev == line || noAccessBetween(line.next, r.prev, r.dst) && noAccessBetween(line.next, r.prev, r.src1, true) && (!hasSrc2 || noAccessBetween(line.next, r.prev, r.src2, true))) {
								// avoid making modifications such as:
								// ---- before ----
								// foo v0.y, v0.x;
								// foo v0.z, v0.y;
								// ---- after ----
								// foo v0.yz, v0.xy;
								if ((line.dst.name != r.src1.name || (line.dst.getComponentMask() & r.src1.getComponentMask()) == 0) &&
									(!hasSrc2 || line.dst.name != r.src2.name || (line.dst.getComponentMask() & r.src2.getComponentMask()) == 0)) {
									line.dst.components += r.dst.components;
									// ---- before ----
									// foo v0.xy, v1.xy(, v2.xy);
									// ... (does not access to v0.zw and write to v1.zx( and to v2.xw))
									// foo v0.zw, v1.zx(, v2.xw);
									// ---- after ----
									// foo v0, v1.xyzx(, v2.xyxw);
									// ...
									line.src1.components += r.src1.components;
									if (hasSrc2) line.src2.components += r.src2.components;
									if (line.dst.components == "xyzw") line.dst.components = "";
									if (line.src1.components == "xyzw") line.src1.components = "";
									if (hasSrc2 && line.src2.components == "xyzw") line.src2.components = "";
									// remove related line from the list
									removeLine(r);
									break OUTER;
								}
							}
						}
					}
				}
			} while (false);
			line = line.next;
		}
	}

	private function removeLine(line:OGSLLine):void {
		if (line == output.lines) {
			output.lines = line.next;
			output.lines.prev = null;
		} else {
			if (line.prev) line.prev.next = line.next;
			if (line.next) line.next.prev = line.prev;
		}
	}

	private function noAccessBetween(from:OGSLLine, to:OGSLLine, field:OGSLField, allowRead:Boolean = false):Boolean {
		var line:OGSLLine = from;
		var name:String = field.name;
		var mask:int = field.getComponentMask();
		while (line != to.next) {
			if (line.hasOverlappedAccessToDst(name, mask)) return false;
			if (!allowRead && line.hasOverlappedAccessToSrc1(name, mask)) return false;
			if (!allowRead && line.hasOverlappedAccessToSrc2(name, mask)) return false;
			line = line.next;
		}
		return true;
	}

	private function complementVaryingOutput():void {
		var line:OGSLLine = output.lines;
		var outputFlag:Vector.<int> = new Vector.<int>(OGSLConstants.MAX_VARYING_REGISTERS, true);
		// find varying components that are not written
		while (line) {
			if (line.dst && line.dst.isVarying()) { // dst is a varying register
				outputFlag[parseInt(line.dst.name.substr(1))] |= line.dst.getComponentMask();
			}
			line = line.next;
		}
		// ... and complement them to all destinations
		line = output.lines;
		while (line) {
			if (line.dst && line.dst.isVarying()) { // dst is a varying register
				var complement:int = ~outputFlag[parseInt(line.dst.name.substr(1))] & 15;
				if ((complement & 8) != 0) line.dst.components += "x";
				if ((complement & 4) != 0) line.dst.components += "y";
				if ((complement & 2) != 0) line.dst.components += "z";
				if ((complement & 1) != 0) line.dst.components += "w";
				if (line.dst.components == "xyzw") line.dst.components = "";
			}
			line = line.next;
		}
	}

	private function willFieldBeReferedAfter(startLine:OGSLLine, field:OGSLField, exceptFirstSrc1:Boolean, exceptFirstSrc2:Boolean):Boolean {
		var mask:int = field.getComponentMask();
		var name:String = field.name;
		var first:Boolean = true;
		var line:OGSLLine = startLine;
		while (line && mask != 0) {
			// check if some components are refered without being written
			if ((!first || !exceptFirstSrc1) && line.hasOverlappedAccessToSrc1(name, mask)) return true;
			if ((!first || !exceptFirstSrc2) && line.hasOverlappedAccessToSrc2(name, mask)) return true;
			// check if some components are written without being refered
			if (line.hasOverlappedAccessToDst(name, mask) && willLineCertainlyBeRun(startLine, line)) mask &= ~line.dst.getComponentMask();
			first = false;
			line = line.next;
		}
		return false;
	}
}

class OGSLOutput {
	public var lines:OGSLLine;
	private var lastLine:OGSLLine;

	public function append(line:OGSLLine):void {
		if (lines) {
			line.prev = lastLine;
			lastLine = lastLine.next = line;
		} else lines = lastLine = line;
	}

	public function print():String {
		var code:String = "";
		var line:OGSLLine = lines;
		while (line) {
			code += line.print() + "\n";
			line = line.next;
		}
		return code;
	}

	public function numLines():int {
		var count:int = 0;
		var line:OGSLLine = lines;
		while (line) {
			count++;
			line = line.next;
		}
		return count;
	}
}

class OGSLLine {
	public var op:String;
	public var dst:OGSLField;
	public var src1:OGSLField;
	public var src2:OGSLField;
	public var prev:OGSLLine;
	public var next:OGSLLine;
	public var toBeRemoved:Boolean;
	public var index:int;

	public function OGSLLine(op:String, dst:OGSLField = null, src1:OGSLField = null, src2:OGSLField = null) {
		if (op == "ifg" || op == "ifl") throw new Error("!?"); // do not use these ops
		this.op = op;
		this.dst = dst;
		this.src1 = src1;
		this.src2 = src2;
	}

	public function hasOverlappedAccessToSrc1(name:String, mask:int):Boolean {
		return src1 && src1.name == name && (src1.getComponentMask() & mask) != 0;
	}

	public function hasOverlappedAccessToSrc2(name:String, mask:int):Boolean {
		if (!src2) return false;
		var f:OGSLField = src2;
		switch (op) {
		case "m44":
			if (f.name == name && (f.getComponentMask() & mask) != 0) return true;
			f = nextRegister(f);
		case "m33":
		case "m34":
			if (f.name == name && (f.getComponentMask() & mask) != 0) return true;
			f = nextRegister(f);
			if (f.name == name && (f.getComponentMask() & mask) != 0) return true;
			f = nextRegister(f);
		default:
			return f.name == name && (f.getComponentMask() & mask) != 0;
		}
	}

	public function hasOverlappedAccessToDst(name:String, mask:int):Boolean {
		return dst && dst.name == name && (dst.getComponentMask() & mask) != 0;
	}

	public function isConditionalOperation():Boolean {
		return op == "ife" || op == "ine" || op == "ifg" || op == "ifl" || op == "els" || op == "eif";
	}

	private function nextRegister(field:OGSLField):OGSLField {
		return new OGSLField(field.name.replace(/[0-9]/g, "") + (parseInt(field.name.replace(/[^0-9]/g, "")) + 1), field.components, field.flags);
	}

	public function print():String {
		var flag:int = 0;
		if (dst) flag |= 1;
		if (src1) flag |= 2;
		if (src2) flag |= 4;
		var code:String = op;
		switch (flag) {
		case 1: // op dst;
		case 4: // op src2;
		case 5: // op dst src2;
			throw new Error("!?"); // NOPE
		case 2: // op src1;
			code += " " + src1.print();
			break;
		case 3: // op dst src1;
			code += " " + dst.print() + ", " + src1.print();
			break;
		case 6: // op src1 src2;
			code += " " + src1.print() + ", " + src2.print();
			break;
		case 7: // op dst src1 src2;
			code += " " + dst.print() + ", " + src1.print() + ", " + src2.print();
			break;
		}
		return code + ";";
	}

	public function reverseCondition():void {
		switch (op) {
		case "ife":
			op = "ine";
			break;
		case "ine":
			op = "ife";
			break;
		default:
			throw new Error("!?");
		}
	}
}

class OGSLField {
	public var name:String;
	public var components:String;
	public var flags:String; // sampler flags

	public function OGSLField(name:String, components:String, flags:String) {
		this.name = name;
		this.components = components;
		this.flags = flags;
	}

	public function print():String {
		return name + (components == "" ? "" : "." + components) + (flags == "" ? "" : " <" + flags + ">");
	}

	public function getComponentMask():int {
		if (components == "") return 15;
		var mask:int = 0;
		if (components.indexOf("x") != -1) mask |= 8;
		if (components.indexOf("y") != -1) mask |= 4;
		if (components.indexOf("z") != -1) mask |= 2;
		if (components.indexOf("w") != -1) mask |= 1;
		return mask; // 0000xyzw
	}

	public function isConstant():Boolean {
		return name.indexOf("vc") == 0 || name.indexOf("fc") == 0;
	}

	public function equals(field:OGSLField):Boolean {
		return field != null && field.name == name && field.components == components;
	}

	public function isTemporary():Boolean {
		return name.indexOf("vt") == 0 || name.indexOf("ft") == 0;
	}

	public function isVarying():Boolean {
		return name.replace(/v[0-9]+/g, "") == "";
	}

	public function getIndex():int {
		var index:String = name.replace(/[^0-9]/g, "");
		if (index == "") throw new Error("!?");
		return parseInt(index);
	}

	public function setIndex(index:int):void {
		name = name.replace(/[0-9]/g, "") + index;
	}

	public function moveComponentsLeft(amount:int):void {
		if (!isTemporary() || amount < 0) throw new Error("!?");
		if (components == "") {
			if ((amount & 3) != 0) throw new Error("!?");
			setIndex(getIndex() - (amount >> 2));
		} else {
			var numComponents:int = components.length;
			var currentIndex:int = getIndex();
			var newIndex:int = -1;
			var c:String = "";
			var i:int;
			switch (numComponents) {
			case 4:
				i = "xyzw".indexOf(components.charAt(3));
				c = "xyzw".charAt(i - amount & 3) + c;
				i = (currentIndex << 2) + i - amount >> 2;
				if (i < 0 || newIndex != -1 && newIndex != i) throw new Error("!?");
				newIndex = i;
			case 3:
				i = "xyzw".indexOf(components.charAt(2));
				c = "xyzw".charAt(i - amount & 3) + c;
				i = (currentIndex << 2) + i - amount >> 2;
				if (i < 0 || newIndex != -1 && newIndex != i) throw new Error("!?");
				newIndex = i;
			case 2:
				i = "xyzw".indexOf(components.charAt(1));
				c = "xyzw".charAt(i - amount & 3) + c;
				i = (currentIndex << 2) + i - amount >> 2;
				if (i < 0 || newIndex != -1 && newIndex != i) throw new Error("!?");
				newIndex = i;
			case 1:
				i = "xyzw".indexOf(components.charAt(0));
				c = "xyzw".charAt(i - amount & 3) + c;
				i = (currentIndex << 2) + i - amount >> 2;
				if (i < 0 || newIndex != -1 && newIndex != i) throw new Error("!?");
				newIndex = i;
				break;
			default:
				throw new Error("!?");
			}
			components = c;
			setIndex(newIndex);
		}
	}
}

class OGSLAccessor {
	public var name:String;
	public var scope:String;
	public var thisAccess:Boolean;
	public var index:String;
	public var components:String;
	public var flags:String;
	public var isAutomaticScalarSwizzlingEnabled:Boolean;

	public function OGSLAccessor(name:String = "", scope:String = "", thisAccess:Boolean = false, index:String = "", components:String = "", flags:String = "") {
		this.name = name;
		this.scope = scope;
		this.thisAccess = thisAccess;
		this.index = index;
		this.components = components;
		this.flags = flags;
	}

	public function clone():OGSLAccessor {
		var accessor:OGSLAccessor = new OGSLAccessor();
		accessor.name = name;
		accessor.scope = scope;
		accessor.thisAccess = thisAccess;
		accessor.index = index;
		accessor.components = components;
		accessor.flags = flags;
		return accessor;
	}

	public function toString():String {
		return scope + "#" + (thisAccess ? "this." : "") + name + (index == "" ? "" : "[" + index + "]") + (components == "" ? "" : "." + components) + (flags == "" ? "" : " <" + flags + ">");
	}

	public function isLiteralNumberAccess():Boolean {
		return name.replace(/-?([0-9]+\.)?[0-9]+/, "") == "";
	}
}

class OGSLEnvironment {
	private var properties:Vector.<OGSLProperty>;
	private var numProperties:int;
	private var tmpCount:int;
	private var vAllocator:OGSLRegisterIndexAllocator;
	private var vtAllocator:OGSLRegisterIndexAllocator;
	private var vcAllocator:OGSLRegisterIndexAllocator;
	private var vaAllocator:OGSLRegisterIndexAllocator;
	private var ftAllocator:OGSLRegisterIndexAllocator;
	private var fcAllocator:OGSLRegisterIndexAllocator;
	private var tmpVector:Vector.<Number>;

	public function OGSLEnvironment() {
		properties = new Vector.<OGSLProperty>(4096, true);
		tmpVector = new Vector.<Number>(4096, true);
		vAllocator = new OGSLRegisterIndexAllocator(OGSLConstants.MAX_VARYING_REGISTERS);
		vtAllocator = new OGSLRegisterIndexAllocator(OGSLConstants.MAX_VERTEX_TEMPORARY_REGISTERS);
		vcAllocator = new OGSLRegisterIndexAllocator(OGSLConstants.MAX_VERTEX_CONSTANT_REGISTERS);
		vaAllocator = new OGSLRegisterIndexAllocator(OGSLConstants.MAX_VERTEX_ATTRIBUTE_REGISTERS);
		ftAllocator = new OGSLRegisterIndexAllocator(OGSLConstants.MAX_FRAGMENT_TEMPORARY_REGISTERS);
		fcAllocator = new OGSLRegisterIndexAllocator(OGSLConstants.MAX_FRAGMENT_CONSTANT_REGISTERS);
		numProperties = 0;
		tmpCount = 0;
	}

	public function access(accessor:OGSLAccessor):OGSLProperty {
		if (accessor == null) return null;
		var scope:String = accessor.scope;
		if (scope == "") return null;
		var name:String = accessor.name;
		if (accessor.thisAccess) {
			switch (getScopeType(scope)) {
			case OGSLConstants.SCOPE_TYPE_VERTEX:
				scope = OGSLConstants.SCOPE_VERTEX;
				break;
			case OGSLConstants.SCOPE_TYPE_FRAGMENT:
				scope = OGSLConstants.SCOPE_FRAGMENT;
				break;
			default:
				throw new Error("!?");
			}
		}
		while (true) {
			for (var i:int = 0; i < numProperties; i++) {
				var prop:OGSLProperty = properties[i];
				if (prop.scope == scope && prop.name == name) return prop;
			}
			if (scope == OGSLConstants.SCOPE_GLOBAL) break; // no property found
			else scope = scope.substr(0, scope.lastIndexOf(".")); // move to parent scope
		}
		return null;
	}

	public function accessFunction(accessor:OGSLAccessor):OGSLFunction {
		var prop:OGSLProperty = access(accessor);
		return prop is OGSLFunction ? OGSLFunction(prop) : null;
	}

	public function accessVariable(accessor:OGSLAccessor):OGSLVariable {
		var prop:OGSLProperty = access(accessor);
		return prop is OGSLVariable ? OGSLVariable(prop) : null;
	}

	public function accessTexture(accessor:OGSLAccessor):OGSLTexture {
		var prop:OGSLProperty = access(accessor);
		return prop is OGSLTexture ? OGSLTexture(prop) : null;
	}

	public function isWritable(accessor:OGSLAccessor):Boolean {
		var v:OGSLVariable = accessVariable(accessor);
		if (!v) throw new Error("!?");
		switch (getScopeType(accessor.scope)) {
		case OGSLConstants.SCOPE_TYPE_VERTEX:
			return v.registerName != "va" && v.registerName != "vc";
		case OGSLConstants.SCOPE_TYPE_FRAGMENT:
			return v.registerName != "v" && v.registerName != "fc";
		default:
			throw new Error("!?");
		}
	}

	public function isConstant(accessor:OGSLAccessor):Boolean {
		if (accessor.isLiteralNumberAccess()) return true;
		var v:OGSLVariable = accessVariable(accessor);
		return v && (v.registerName == "vc" || v.registerName == "fc");
	}

	public function getVariableType(accessor:OGSLAccessor):String {
		if (accessor.isLiteralNumberAccess()) return OGSLConstants.TYPE_FLOAT; // literal number
		var prop:OGSLProperty = access(accessor);
		if (prop == null || prop.type == "function" || prop.type == "texture") throw new Error("no such variable: " + accessor.name);
		if (prop.type == "void") throw new Error("you cannot access to void");
		var v:OGSLVariable = OGSLVariable(prop);
		if (accessor.index != "" && !(parseInt(accessor.index) >= 0)) throw new Error("!?");
		var index:int = accessor.index == "" ? -1 : parseInt(accessor.index);
		switch (v.type) {
		case OGSLConstants.TYPE_FLOAT:
			if (accessor.components.replace(/[x]/g, "") != "") throw new Error("components " + accessor.components + " is not available in float");
			if (index != -1) throw new Error("index " + index + " is not available in float");
			switch (accessor.components.length) {
			case 0:
				return OGSLConstants.TYPE_FLOAT;
			case 1:
				return OGSLConstants.TYPE_FLOAT;
			case 2:
				return OGSLConstants.TYPE_VEC2;
			case 3:
				return OGSLConstants.TYPE_VEC3;
			case 4:
				return OGSLConstants.TYPE_VEC4;
			}
			break;
		case OGSLConstants.TYPE_VEC2:
			if (accessor.components.replace(/[xy]/g, "") != "") throw new Error("components " + accessor.components + " is not available in vec2");
			if (index != -1) throw new Error("index " + index + " is not available in vec2");
			switch (accessor.components.length) {
			case 0:
				return OGSLConstants.TYPE_VEC2;
			case 1:
				return OGSLConstants.TYPE_FLOAT;
			case 2:
				return OGSLConstants.TYPE_VEC2;
			case 3:
				return OGSLConstants.TYPE_VEC3;
			case 4:
				return OGSLConstants.TYPE_VEC4;
			}
			break;
		case OGSLConstants.TYPE_VEC3:
			if (accessor.components.replace(/[xyz]/g, "") != "") throw new Error("components " + accessor.components + " is not available in vec3");
			if (index != -1) throw new Error("index " + index + " is not available in vec3");
			switch (accessor.components.length) {
			case 0:
				return OGSLConstants.TYPE_VEC3;
			case 1:
				return OGSLConstants.TYPE_FLOAT;
			case 2:
				return OGSLConstants.TYPE_VEC2;
			case 3:
				return OGSLConstants.TYPE_VEC3;
			case 4:
				return OGSLConstants.TYPE_VEC4;
			}
			break;
		case OGSLConstants.TYPE_VEC4:
			if (index != -1) throw new Error("index " + index + " is not available in vec4");
			switch (accessor.components.length) {
			case 0:
				return OGSLConstants.TYPE_VEC4;
			case 1:
				return OGSLConstants.TYPE_FLOAT;
			case 2:
				return OGSLConstants.TYPE_VEC2;
			case 3:
				return OGSLConstants.TYPE_VEC3;
			case 4:
				return OGSLConstants.TYPE_VEC4;
			}
			break;
		case OGSLConstants.TYPE_MAT3X4:
			if (index == -1) {
				if (accessor.components != "") throw new Error("components " + accessor.components + " with no index is not available in mat3x4");
				return OGSLConstants.TYPE_MAT3X4;
			}
			if (index >= 3) throw new Error("index " + index + " is not available in mat3x4");
			switch (accessor.components.length) {
			case 0:
				return OGSLConstants.TYPE_VEC4;
			case 1:
				return OGSLConstants.TYPE_FLOAT;
			case 2:
				return OGSLConstants.TYPE_VEC2;
			case 3:
				return OGSLConstants.TYPE_VEC3;
			case 4:
				return OGSLConstants.TYPE_VEC4;
			}
			break;
		case OGSLConstants.TYPE_MAT4X4:
			if (index == -1) {
				if (accessor.components != "") throw new Error("components " + accessor.components + " with no index is not available in mat4x4");
				return OGSLConstants.TYPE_MAT4X4;
			}
			if (index >= 4) throw new Error("index " + index + " is not available in mat4x4");
			switch (accessor.components.length) {
			case 0:
				return OGSLConstants.TYPE_VEC4;
			case 1:
				return OGSLConstants.TYPE_FLOAT;
			case 2:
				return OGSLConstants.TYPE_VEC2;
			case 3:
				return OGSLConstants.TYPE_VEC3;
			case 4:
				return OGSLConstants.TYPE_VEC4;
			}
			break;
		}
		throw new Error("!?");
	}

	public function getScopeType(scope:String):String {
		if (scope.indexOf(OGSLConstants.SCOPE_VERTEX) == 0) return OGSLConstants.SCOPE_TYPE_VERTEX;
		if (scope.indexOf(OGSLConstants.SCOPE_FRAGMENT) == 0) return OGSLConstants.SCOPE_TYPE_FRAGMENT;
		if (scope.indexOf(OGSLConstants.SCOPE_GLOBAL) == 0) return OGSLConstants.SCOPE_TYPE_GLOBAL;
		throw new Error("!?");
	}

	public function getScopeByProgramType(programType:String):String {
		if (programType == OGSLConstants.PROGRAM_TYPE_VERTEX) return OGSLConstants.SCOPE_VERTEX;
		if (programType == OGSLConstants.PROGRAM_TYPE_FRAGMENT) return OGSLConstants.SCOPE_FRAGMENT;
		throw new Error("!?");
	}

	public function getVertexBufferIndexMap():Object {
		var map:Object = new Object();
		for (var i:int = 0; i < numProperties; i++) {
			if (isVertexAttribute(properties[i])) {
				var v:OGSLVariable = OGSLVariable(properties[i]);
				map[v.name] = v.registerIndex;
			}
		}
		return map;
	}

	public function getVertexConstantsIndexMap():Object {
		var map:Object = new Object();
		for (var i:int = 0; i < numProperties; i++) {
			if (isVertexConstants(properties[i]) && !isLiteralNumberVariable(properties[i])) {
				var v:OGSLVariable = OGSLVariable(properties[i]);
				map[v.name] = v.registerIndex;
			}
		}
		return map;
	}

	public function getFragmentConstantsIndexMap():Object {
		var map:Object = new Object();
		for (var i:int = 0; i < numProperties; i++) {
			if (isFragmentConstants(properties[i]) && !isLiteralNumberVariable(properties[i])) {
				var v:OGSLVariable = OGSLVariable(properties[i]);
				map[v.name] = v.registerIndex;
			}
		}
		return map;
	}

	public function getTextureIndexMap():Object {
		var map:Object = new Object();
		for (var i:int = 0; i < numProperties; i++) {
			if (properties[i] is OGSLTexture) {
				var t:OGSLTexture = OGSLTexture(properties[i]);
				map[t.name] = t.textureIndex;
			}
		}
		return map;
	}

	public function getDefaultConstantsDataVertex():Vector.<Number> {
		return getDefaultConstantsDataAt(OGSLConstants.SCOPE_VERTEX);
	}

	public function getDefaultConstantsDataFragment():Vector.<Number> {
		return getDefaultConstantsDataAt(OGSLConstants.SCOPE_FRAGMENT);
	}

	private function getDefaultConstantsDataAt(scope:String):Vector.<Number> {
		clearTmpVector();
		var maxRegIndex:int = 0;
		var numRegisters:int = 0;
		for (var i:int = 0; i < numProperties; i++) {
			if (properties[i].scope == scope && isLiteralNumberVariable(properties[i])) {
				var v:OGSLVariable = OGSLVariable(properties[i]);
				var offset:int = v.registerIndex * 5;
				if (tmpVector[offset] == 0) {
					tmpVector[offset] = 1;
					numRegisters++;
				}
				tmpVector[offset + 1 + "xyzw".indexOf(v.registerComponentOrder.charAt(0))] = parseFloat(v.name.replace("%num", ""));
				if (v.registerIndex > maxRegIndex) maxRegIndex = v.registerIndex;
			}
		}
		var data:Vector.<Number> = new Vector.<Number>(numRegisters * 5, true);
		var index:int = 0;
		for (i = 0; i <= maxRegIndex; i++) {
			if (tmpVector[i * 5] == 1) {
				data[index] = i;
				data[index + 1] = tmpVector[i * 5 + 1];
				data[index + 2] = tmpVector[i * 5 + 2];
				data[index + 3] = tmpVector[i * 5 + 3];
				data[index + 4] = tmpVector[i * 5 + 4];
				index += 5;
			}
		}
		return data;
	}

	private function clearTmpVector():void {
		for (var i:int = 0; i < tmpVector.length; i++) tmpVector[i] = 0;
	}

	private function createLiteralNumberConstantAccess(number:Number, scope:String, isASSEnabled:Boolean, components:String):OGSLAccessor {
		var isVertex:Boolean;
		switch (getScopeType(scope)) {
		case OGSLConstants.SCOPE_TYPE_VERTEX:
			isVertex = true;
			break;
		case OGSLConstants.SCOPE_TYPE_FRAGMENT:
			isVertex = false;
			break;
		default:
			throw new Error("!?");
		}
		var accessor:OGSLAccessor = new OGSLAccessor("%num" + number, isVertex ? OGSLConstants.SCOPE_VERTEX : OGSLConstants.SCOPE_FRAGMENT);
		if (isASSEnabled) {
			accessor.isAutomaticScalarSwizzlingEnabled = true;
			accessor.components = components;
		}
		if (accessVariable(accessor)) return accessor;
		var v:OGSLVariable = new OGSLVariable();
		v.name = "%num" + number;
		v.scope = isVertex ? OGSLConstants.SCOPE_VERTEX : OGSLConstants.SCOPE_FRAGMENT;
		v.registerComponentOrder = "xyzw";
		v.type = OGSLConstants.TYPE_FLOAT;
		v.registerName = isVertex ? "vc" : "fc";
		addVariable(v);
		return accessor;
	}

	public function getField(accessor:OGSLAccessor):OGSLField {
		if (accessor.isLiteralNumberAccess()) {
			return getField(createLiteralNumberConstantAccess(parseFloat(accessor.name), accessor.scope, accessor.isAutomaticScalarSwizzlingEnabled, accessor.components));
		}
		var prop:OGSLProperty = access(accessor);
		if (prop is OGSLTexture) {
			var t:OGSLTexture = OGSLTexture(prop);
			var index:int = t.textureIndex;
			return new OGSLField("fs" + index, "", accessor.flags);
		} else if (prop is OGSLVariable) {
			var v:OGSLVariable = OGSLVariable(prop);
			var name:String = v.registerName;
			if (v.registerIndex == -1) {
				if (!accessor.isAutomaticScalarSwizzlingEnabled) name += accessor.index;
			} else {
				if (accessor.index == "" || accessor.isAutomaticScalarSwizzlingEnabled) name += v.registerIndex;
				else name += v.registerIndex + parseInt(accessor.index);
			}
			var components:String = accessor.components == "" ? v.getDefaultComponents() : accessor.components;
			var o1:String = v.registerComponentOrder.charAt(0);
			var o2:String = v.registerComponentOrder.charAt(1);
			var o3:String = v.registerComponentOrder.charAt(2);
			var o4:String = v.registerComponentOrder.charAt(3);
			components = components.toUpperCase().replace(/X/g, o1).replace(/Y/g, o2).replace(/Z/g, o3).replace(/W/g, o4);
			if (components == "xyzw") components = "";
			return new OGSLField(name, components, "");
		} else throw new Error("!?");
	}

	public function addFunction(func:OGSLFunction):void {
		if (func.type != "function") throw new Error("!?");
		addProperty(func);
	}

	public function addTexture(tex:OGSLTexture):void {
		if (tex.type != "texture") throw new Error("!?");
		addProperty(tex);
	}

	public function addVariable(variable:OGSLVariable):void {
		switch (variable.registerName) {
		case "v":
			vAllocator.allocate(variable);
			break;
		case "vt":
			vtAllocator.allocate(variable, true);
			break;
		case "vc":
			vcAllocator.allocate(variable, variable.name.indexOf("%num") == 0);
			break;
		case "va":
			vaAllocator.allocate(variable);
			break;
		case "ft":
			ftAllocator.allocate(variable, true);
			break;
		case "fc":
			fcAllocator.allocate(variable, variable.name.indexOf("%num") == 0);
			break;
		}
		addProperty(variable);
	}

	private function isTempVariable(prop:OGSLProperty):Boolean {
		return prop != null && prop.name.indexOf("%tmp") == 0;
	}

	private function isLiteralNumberVariable(prop:OGSLProperty):Boolean {
		return prop != null && prop.name.indexOf("%num") == 0;
	}

	private function isVertexAttribute(prop:OGSLProperty):Boolean {
		return prop != null && prop is OGSLVariable && OGSLVariable(prop).registerName == "va";
	}

	private function isVertexConstants(prop:OGSLProperty):Boolean {
		return prop is OGSLVariable && OGSLVariable(prop).registerName == "vc";
	}

	private function isFragmentConstants(prop:OGSLProperty):Boolean {
		return prop is OGSLVariable && OGSLVariable(prop).registerName == "fc";
	}

	public function createTempVariable(type:String, scope:String):OGSLAccessor {
		var tmp:OGSLVariable = new OGSLVariable();
		tmp.scope = scope;
		tmp.name = "%tmp" + ++tmpCount;
		tmp.type = type;
		switch (getScopeType(scope)) {
		case OGSLConstants.SCOPE_TYPE_VERTEX:
			tmp.registerName = "vt";
			break;
		case OGSLConstants.SCOPE_TYPE_FRAGMENT:
			tmp.registerName = "ft";
			break;
		default:
			throw new Error("!?");
		}
		addVariable(tmp);
		return new OGSLAccessor(tmp.name, tmp.scope);
	}

	public function destroyTempVariablesAt(scope:String):void {
		for (var i:int = 0; i < numProperties; i++) {
			if (properties[i].scope == scope && isTempVariable(properties[i])) {
				removeProperty(i--);
			}
		}
	}

	public function destroyVariablesAt(scope:String):void {
		for (var i:int = 0; i < numProperties; i++) {
			if (properties[i].scope == scope) removeProperty(i--);
		}
	}

	public function combineComponentAccess(lhs:OGSLAccessor, additionalIndex:String, additionalComponents:String):void {
		var i1:String = lhs.index;
		var c1:String = lhs.components;
		var i2:String = additionalIndex;
		var c2:String = additionalComponents;
		if (lhs.isLiteralNumberAccess()) throw new Error("invalid variable access: " + lhs + "[" + i2 + "]");
		var flag:int = 0;
		if (i1 != "") flag |= 1;
		if (c1 != "") flag |= 2;
		if (i2 != "") flag |= 4;
		if (c2 != "") flag |= 8;
		switch (flag) {
		case 0:
		case 1:
		case 2:
		case 3: // no additional accesses
			return;
		case 4:
		case 8:
		case 12: // additional accesses only
			lhs.index = i2;
			lhs.components = c2;
			return;
		case 5: // like (foo[0])[0]
			if (parseInt(i2) >= 4) throw new Error("invalid variable access: " + lhs + "[" + i2 + "]");
			lhs.components = "xyzw".charAt(parseInt(i2));
			return;
		case 6: // like (foo.xyz)[0]
		case 7: // like (foo[0].xyz)[0]
		case 13: // like (foo[0])[0].xyz
		case 14: // like (foo.xyz)[0].xyz
		case 15: // like (foo[0].xyz)[0].xyz
			throw new Error("invalid variable access: " + lhs + "[" + i2 + "]" + (c2 == "" ? "" : "." + c2));
		case 9: // like (foo[0]).xyz
			lhs.components = c2;
			return;
		case 10: // like (foo.xyz).xyz
		case 11: // like (foo[0].xyz).xyz
			switch (c1.length) {
			case 1:
				if (c2.replace(/[x]/g, "") != "") throw new Error("invalid variable access: " + lhs + "." + c2);
				lhs.components = c2.toUpperCase().replace(/X/g, c1.charAt(0));
				return;
			case 2:
				if (c2.replace(/[xy]/g, "") != "") throw new Error("invalid variable access: " + lhs + "." + c2);
				lhs.components = c2.toUpperCase().replace(/X/g, c1.charAt(0)).replace(/Y/g, c1.charAt(1));
				return;
			case 3:
				if (c2.replace(/[xyz]/g, "") != "") throw new Error("invalid variable access: " + lhs + "." + c2);
				lhs.components = c2.toUpperCase().replace(/X/g, c1.charAt(0)).replace(/Y/g, c1.charAt(1)).replace(/Z/g, c1.charAt(2));
				return;
			case 4:
				lhs.components = c2.toUpperCase().replace(/X/g, c1.charAt(0)).replace(/Y/g, c1.charAt(1)).replace(/Z/g, c1.charAt(2)).replace(/W/g, c1.charAt(3));
				return;
			}
		}
		throw new Error("!?");
	}

	private function addProperty(property:OGSLProperty):void {
		for (var i:int = 0; i < numProperties; i++) {
			var prop:OGSLProperty = properties[i];
			if (prop.scope == property.scope && prop.name == property.name) throw new Error("duplicate properties: " + prop.scope + "#" + prop.name);
		}
		properties[numProperties++] = property;
	}

	private function removeProperty(index:int):void {
		var prop:OGSLProperty = properties[index];
		if (prop is OGSLVariable) {
			var v:OGSLVariable = OGSLVariable(prop);
			switch (v.registerName) {
			case "vt":
				vtAllocator.release(v, true);
				break;
			case "ft":
				ftAllocator.release(v, true);
				break;
			default:
				throw new Error("!?");
			}
		}
		properties[index] = null;
		properties[index] = properties[--numProperties];
	}
}

class OGSLRegisterIndexAllocator {
	private var numRegisters:int;
	private var usage:Vector.<int>;

	public function OGSLRegisterIndexAllocator(numRegisters:int) {
		this.numRegisters = numRegisters;
		usage = new Vector.<int>(this.numRegisters, true);
	}

	public function allocate(v:OGSLVariable, pack:Boolean = false):void {
		switch (v.type) {
		case OGSLConstants.TYPE_FLOAT:
			if (pack) allocateFloat(v);
			else allocateVecs(v, 1);
			break;
		case OGSLConstants.TYPE_VEC2:
		case OGSLConstants.TYPE_VEC3:
		case OGSLConstants.TYPE_VEC4:
			allocateVecs(v, 1);
			break;
		case OGSLConstants.TYPE_MAT3X4:
			allocateVecs(v, 3);
			break;
		case OGSLConstants.TYPE_MAT4X4:
			allocateVecs(v, 4);
			break;
		default:
			throw new Error("!?");
		}
	}

	public function release(v:OGSLVariable, packed:Boolean = false):void {
		switch (v.type) {
		case OGSLConstants.TYPE_FLOAT:
			if (packed) releaseFloat(v);
			else releaseVecs(v, 1);
			break;
		case OGSLConstants.TYPE_VEC2:
		case OGSLConstants.TYPE_VEC3:
		case OGSLConstants.TYPE_VEC4:
			releaseVecs(v, 1);
			break;
		case OGSLConstants.TYPE_MAT3X4:
			releaseVecs(v, 3);
			break;
		case OGSLConstants.TYPE_MAT4X4:
			releaseVecs(v, 4);
			break;
		default:
			throw new Error("!?");
		}
	}

	public function releaseAll():void {
		for (var i:int = 0; i < numRegisters; i++) {
			usage[i] = 0;
		}
	}

	private function allocateFloat(v:OGSLVariable):void {
		for (var i:int = 0; i < numRegisters; i++) {
			if ((usage[i] & 8) == 0) {
				usage[i] |= 8;
				v.registerIndex = i;
				v.registerComponentOrder = "xyzw";
				return;
			}
			if ((usage[i] & 4) == 0) {
				usage[i] |= 4;
				v.registerIndex = i;
				v.registerComponentOrder = "yxzw";
				return;
			}
			if ((usage[i] & 2) == 0) {
				usage[i] |= 2;
				v.registerIndex = i;
				v.registerComponentOrder = "zxyw";
				return;
			}
			if ((usage[i] & 1) == 0) {
				usage[i] |= 1;
				v.registerIndex = i;
				v.registerComponentOrder = "wxyz";
				return;
			}
		}
		throw new Error("registers overflow");
	}

	private function allocateVecs(v:OGSLVariable, num:int):void {
		var unusedCount:int = 0;
		for (var i:int = 0; i < numRegisters; i++) {
			if (usage[i] != 0) unusedCount = 0;
			else unusedCount++;
			if (unusedCount == num) {
				for (var j:int = 0; j < num; j++) {
					usage[i - j] = 15;
				}
				v.registerIndex = i - num + 1;
				v.registerComponentOrder = "xyzw";
				return;
			}
		}
		throw new Error("registers overflow");
	}

	private function releaseFloat(v:OGSLVariable):void {
		switch (v.registerComponentOrder.charAt(0)) {
		case "x":
			usage[v.registerIndex] &= ~8;
			break;
		case "y":
			usage[v.registerIndex] &= ~4;
			break;
		case "z":
			usage[v.registerIndex] &= ~2;
			break;
		case "w":
			usage[v.registerIndex] &= ~1;
			break;
		default:
			throw new Error("!?");
		}
	}

	private function releaseVecs(v:OGSLVariable, num:int):void {
		for (var i:int = 0; i < num; i++) {
			usage[v.registerIndex + i] = 0;
		}
	}
}

class OGSLProperty {
	public var name:String;
	public var type:String;
	public var scope:String;

	public function toString():String {
		return scope + "#" + name + ":" + type;
	}
}

class OGSLFunction extends OGSLProperty {
	public var rpn:String;
	public var args:String;
	public var returnType:String;

	public function complementOutput():void {
		if (name != "main") throw new Error("!?");
		if (rpn == "") rpn = "%output 0 0 0 1 vec4 4 () = ;";
		else rpn = "output 0 0 0 1 vec4 4 () = ; " + rpn + " %output output = ;";
	}
}

class OGSLVariable extends OGSLProperty {
	public var registerName:String;
	public var registerIndex:int;
	public var registerComponentOrder:String;

	public function getDefaultComponents():String {
		switch (type) {
		case OGSLConstants.TYPE_FLOAT:
			return "x";
		case OGSLConstants.TYPE_VEC2:
			return "xy";
		case OGSLConstants.TYPE_VEC3:
			return "xyz";
		case OGSLConstants.TYPE_VEC4:
		case OGSLConstants.TYPE_MAT3X4:
		case OGSLConstants.TYPE_MAT4X4:
			return "";
		default:
			throw new Error("!?");
		}
	}
}

class OGSLTexture extends OGSLProperty {
	public var textureIndex:int;
}

class OGSLNode {
	public function OGSLNode() {
	}
}

class OGSLExpressionNode extends OGSLNode {
	public var next:OGSLExpressionNode;

	public function dumpRPN():String {
		throw new Error("!?");
	}
}

class OGSLAssignmentExpressionNode extends OGSLExpressionNode {
	public var lhs:OGSLExpressionNode;
	public var operator:String;
	public var rhs:OGSLExpressionNode;

	public override function dumpRPN():String {
		return lhs.dumpRPN() + " " + rhs.dumpRPN() + " " + operator;
	}
}

class OGSLBinaryExpressionNode extends OGSLExpressionNode {
	public var lhs:OGSLExpressionNode;
	public var operator:String;
	public var rhs:OGSLExpressionNode;

	public override function dumpRPN():String {
		return lhs.dumpRPN() + " " + rhs.dumpRPN() + " " + operator;
	}
}

class OGSLUnaryExpressionNode extends OGSLExpressionNode {
	public var operator:String;
	public var rhs:OGSLExpressionNode;

	public override function dumpRPN():String {
		return rhs.dumpRPN() + " " + operator;
	}
}

class OGSLFunctionCallExpressionNode extends OGSLExpressionNode {
	public var name:OGSLIdentifierExpressionNode;
	public var args:OGSLArgumentsNode;

	public override function dumpRPN():String {
		var rpn:String = "";
		var arg:OGSLExpressionNode = args.expressions;
		var numArgs:int = 0;
		while (arg) {
			rpn += (rpn == "" ? "" : " ") + arg.dumpRPN();
			arg = arg.next;
			numArgs++;
		}
		return (rpn == "" ? "" : rpn + " ") + name.dumpRPN() + " " + numArgs + " ()";
	}
}

class OGSLArgumentsNode extends OGSLNode {
	public var expressions:OGSLExpressionNode;
	private var lastExpression:OGSLExpressionNode;

	public function addExpression(expression:OGSLExpressionNode):void {
		if (expressions) lastExpression = lastExpression.next = expression;
		else lastExpression = expressions = expression;
	}
}

class OGSLIdentifierExpressionNode extends OGSLExpressionNode {
	public var name:String;

	public override function dumpRPN():String {
		return name;
	}
}

class OGSLLiteralNumberExpressionNode extends OGSLExpressionNode {
	public var number:String;

	public override function dumpRPN():String {
		return number;
	}
}

class OGSLThisAccessExpressionNode extends OGSLIdentifierExpressionNode {
	public override function dumpRPN():String {
		return name + " this";
	}
}

class OGSLRegisterExpressionNode extends OGSLExpressionNode {
	public var lhs:OGSLExpressionNode;
	public var index:String;
	public var components:String;

	public override function dumpRPN():String {
		return lhs.dumpRPN() + " " + index + ":" + components + " .";
	}
}

class OGSLStatementNode extends OGSLNode {
	public var next:OGSLStatementNode;

	public function dumpRPN():String {
		throw new Error("!?");
	}
}

// for inlined looping
class OGSLRPNStatementNode extends OGSLStatementNode {
	public var rpn:String;

	public function OGSLRPNStatementNode(rpn:String) {
		this.rpn = rpn;
	}

	public override function dumpRPN():String {
		return rpn;
	}
}

// "var" inside functions
class OGSLVariableDefinitionStatementNode extends OGSLStatementNode {
	public var variables:OGSLVariableDefinitionAssignmentNode;
	private var lastVariable:OGSLVariableDefinitionAssignmentNode;

	public function addVariable(variable:OGSLVariableDefinitionAssignmentNode):void {
		if (variables) lastVariable = lastVariable.next = variable;
		else lastVariable = variables = variable;
	}

	public override function dumpRPN():String {
		var rpn:String = "";
		var variable:OGSLVariableDefinitionAssignmentNode = variables;
		while (variable) {
			rpn += (rpn == "" ? "" : " ; ") + variable.variable.name + " " + variable.variable.type + " var";
			if (variable.assignment) rpn += " " + variable.variable.name + " " + variable.assignment.dumpRPN() + " =";
			variable = variable.next;
		}
		return rpn;
	}
}

// "var"/"const" outside functions
class OGSLVariableDefinitionInstructionNode extends OGSLNode {
	public var variables:OGSLVariableDefinitionNode;
	public var type:String;
	public var next:OGSLVariableDefinitionInstructionNode;
	private var lastVariable:OGSLVariableDefinitionNode;

	public function addVariable(variable:OGSLVariableDefinitionNode):void {
		if (variables) lastVariable = lastVariable.next = variable;
		else lastVariable = variables = variable;
	}
}

class OGSLAssignmentExpressionStatementNode extends OGSLStatementNode {
	public var expression:OGSLExpressionNode;

	public override function dumpRPN():String {
		return expression.dumpRPN();
	}
}

class OGSLReturnStatementNode extends OGSLStatementNode {
	public var expression:OGSLExpressionNode;

	public override function dumpRPN():String {
		return expression == null ? "0 return" : expression.dumpRPN() + " 1 return";
	}
}

class OGSLDiscardStatementNode extends OGSLStatementNode {
	public override function dumpRPN():String {
		return "discard";
	}
}

class OGSLIfStatementNode extends OGSLStatementNode {
	public var condition:OGSLExpressionNode;

	public override function dumpRPN():String {
		return condition.dumpRPN() + " if"; // if (condition != 0)
	}
}

class OGSLElseStatementNode extends OGSLStatementNode {
	public override function dumpRPN():String {
		return "else";
	}
}

class OGSLEndIfStatementNode extends OGSLStatementNode {
	public override function dumpRPN():String {
		return "endif";
	}
}

class OGSLVariableDefinitionAssignmentNode extends OGSLNode {
	public var next:OGSLVariableDefinitionAssignmentNode;
	public var variable:OGSLVariableDefinitionNode;
	public var assignment:OGSLExpressionNode;
}

class OGSLMainNode extends OGSLNode {
	public var definitions:OGSLVariableDefinitionInstructionNode;
	private var lastDefinition:OGSLVariableDefinitionInstructionNode;
	public var programs:OGSLProgramNode;
	private var lastProgram:OGSLProgramNode;

	public function addDefinition(definition:OGSLVariableDefinitionInstructionNode):void {
		if (definitions) lastDefinition = lastDefinition.next = definition;
		else lastDefinition = definitions = definition;
	}

	public function addProgram(program:OGSLProgramNode):void {
		if (programs) lastProgram = lastProgram.next = program;
		else lastProgram = programs = program;
	}
}

class OGSLProgramNode extends OGSLNode {
	public var next:OGSLProgramNode;
	public var type:String;
	public var definitions:OGSLVariableDefinitionInstructionNode;
	private var lastDefinition:OGSLVariableDefinitionInstructionNode;
	public var functions:OGSLFunctionNode;
	private var lastFunction:OGSLFunctionNode;

	public function addDefinition(definition:OGSLVariableDefinitionInstructionNode):void {
		if (definitions) lastDefinition = lastDefinition.next = definition;
		else lastDefinition = definitions = definition;
	}

	public function addFunction(func:OGSLFunctionNode):void {
		if (functions) lastFunction = lastFunction.next = func;
		else lastFunction = functions = func;
	}
}

class OGSLFunctionNode extends OGSLNode {
	public var next:OGSLFunctionNode;
	public var name:String;
	public var args:OGSLArgumentsDefinitionNode;
	public var returnType:String;
	public var statements:OGSLStatementNode;
	private var lastStatement:OGSLStatementNode;

	public function addStatement(statement:OGSLStatementNode):void {
		if (lastStatement is OGSLReturnStatementNode) throw new Error("return statement must be the last statement in a function");
		if (statements) lastStatement = lastStatement.next = statement;
		else lastStatement = statements = statement;
	}

	public function dumpRPN():String {
		var rpn:String = "";
		var statement:OGSLStatementNode = statements;
		while (statement) {
			rpn += (rpn == "" ? "" : " ") + statement.dumpRPN() + " ;";
			statement = statement.next;
		}
		return rpn;
	}

	public function addReturnStatementIfRequired():void {
		if (!(lastStatement is OGSLReturnStatementNode)) addStatement(new OGSLReturnStatementNode());
	}
}

class OGSLArgumentsDefinitionNode extends OGSLNode {
	public var variables:OGSLVariableDefinitionNode;
	private var lastVariable:OGSLVariableDefinitionNode;

	public function addVariable(variable:OGSLVariableDefinitionNode):void {
		if (variables) lastVariable = lastVariable.next = variable;
		else lastVariable = variables = variable;
	}
}

class OGSLVariableDefinitionNode extends OGSLNode {
	public var next:OGSLVariableDefinitionNode;
	public var name:String;
	public var type:String;
	public var passByReference:Boolean;
}

class StringReader {
	private var data:String;
	private var position:int;
	private var length:int;

	public function StringReader(data:String) {
		this.data = data;
		position = 0;
		length = this.data.length;
	}

	public function read():String {
		if (isEnd()) return "\0";
		return data.charAt(position++);
	}

	public function next(count:int = 1):String {
		var pos:int = position - 1 + count;
		if (pos >= length) return "\0";
		return data.charAt(pos);
	}

	public function isEnd():Boolean {
		return position == length;
	}
}

class TokenReader {
	private var tokens:OGSLToken;

	public function TokenReader(tokens:OGSLToken) {
		this.tokens = tokens;
	}

	public function read(expected:String = null):String {
		var data:String = tokens.data;
		if (expected && data != expected) throw new Error("unexpected token: \"" + data + "\", expected: \"" + expected + "\"");
		tokens = tokens.next;
		return data;
	}

	public function readNumber():String {
		var data:String = tokens.data;
		if (tokens.type != OGSLToken.NUMBER) throw new Error("unexpected token: \"" + data + "\", expected number");
		tokens = tokens.next;
		return data;
	}

	public function readInteger():String {
		var data:String = tokens.data;
		if (tokens.type != OGSLToken.NUMBER || data.indexOf(".") != -1) throw new Error("unexpected token: \"" + data + "\", expected integer");
		tokens = tokens.next;
		return data;
	}

	public function readIdentifier():String {
		var data:String = tokens.data;
		if (tokens.type != OGSLToken.IDENTIFIER) throw new Error("unexpected token: \"" + data + "\", expected identifier");
		tokens = tokens.next;
		return data;
	}

	public function isNextType(type:int):Boolean {
		return tokens.type == type;
	}

	public function isNext(data:String):Boolean {
		return tokens.data == data;
	}

	public function next():String {
		return tokens.data;
	}

	public function isEnd():Boolean {
		return tokens.type == OGSLToken.EOF;
	}
}
