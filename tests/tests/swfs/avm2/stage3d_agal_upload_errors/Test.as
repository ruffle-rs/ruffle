package {
	import AGALMiniAssembler;

	import flash.display.MovieClip;
	import flash.display.Stage3D;
	import flash.display3D.Context3D;
	import flash.display3D.Context3DProgramType;
	import flash.display3D.Program3D;
	import flash.events.Event;
	import flash.utils.ByteArray;

	public class Test extends MovieClip {
		private var context:Context3D;
		private var validVertex:ByteArray;

		public function Test() {
			stage.stage3Ds[0].addEventListener(Event.CONTEXT3D_CREATE, contextCreated);
			stage.stage3Ds[0].requestContext3D();
		}

		private function tryUpload(label:String, vertex:ByteArray, fragment:ByteArray):void {
			var program:Program3D = context.createProgram();
			try {
				program.upload(vertex, fragment);
				trace(label + ": uploaded");
			} catch (e:Error) {
				trace(label + ": " + e.getStackTrace());
			}
		}

		// Write the AGAL fragment shader header
		private function writeFragmentHeader(b:ByteArray):void {
			b.writeByte(0xa0);        // magic
			b.writeUnsignedInt(1);    // version
			b.writeByte(0xa1);        // shader type marker
			b.writeByte(0x01);        // fragment shader
		}

		// Write the AGAL vertex shader header
		private function writeVertexHeader(b:ByteArray):void {
			b.writeByte(0xa0);        // magic
			b.writeUnsignedInt(1);    // version
			b.writeByte(0xa1);        // shader type marker
			b.writeByte(0x00);        // vertex shader
		}

		// Write a mov instruction: mov <dest>, <source>
		// dest is encoded as: reg_num(16) | write_mask(4) | reg_type(4) | pad(8)
		// source is encoded as two u32s:
		//   low:  reg_num(16) | indirect_offset(8) | swizzle(8)
		//   high: register_type(4) | pad(4) | index_type(4) | pad(4) | index_select(2) | pad(14) | direct_mode(1)
		private function writeMov(b:ByteArray, destType:int, destReg:int,
				srcType:int, srcReg:int, indirect:Boolean = false,
				indexType:int = 0, indexSelect:int = 0, indirectOffset:int = 0):void {
			b.writeUnsignedInt(0x00); // opcode: mov
			// dest
			b.writeUnsignedInt(destReg | (0xF << 16) | (destType << 24));
			// source1 low: reg_num | indirect_offset | swizzle
			b.writeUnsignedInt(srcReg | (indirectOffset << 16) | (0xE4 << 24));
			// source1 high: register_type | index_type | index_select | direct_mode
			var highBits:uint = srcType | (indexType << 8) | (indexSelect << 16);
			if (indirect) highBits |= (1 << 31);
			b.writeUnsignedInt(highBits);
			// source2: unused for mov
			b.writeUnsignedInt(0);
			b.writeUnsignedInt(0);
		}

		// Register type constants
		private static const REG_ATTRIBUTE:int = 0;
		private static const REG_CONSTANT:int = 1;
		private static const REG_TEMPORARY:int = 2;
		private static const REG_OUTPUT:int = 3;
		private static const REG_VARYING:int = 4;
		private static const REG_SAMPLER:int = 5;
		private static const REG_FRAGMENT:int = 6;

		// Build a minimal valid AGAL fragment shader (header + "mov oc, fc0")
		private function makeValidFragment():ByteArray {
			var b:ByteArray = new ByteArray();
			b.endian = "littleEndian";
			writeFragmentHeader(b);
			writeMov(b, REG_OUTPUT, 0, REG_CONSTANT, 0);
			b.position = 0;
			return b;
		}

		private function contextCreated(event:Event):void {
			var stage3d:Stage3D = event.target as Stage3D;
			context = stage3d.context3D;
			context.configureBackBuffer(200, 200, 0, false);

			// Build a valid vertex shader with AGALMiniAssembler
			var vertexAssembly:AGALMiniAssembler = new AGALMiniAssembler(false);
			vertexAssembly.assemble(Context3DProgramType.VERTEX,
				"mov op, va0\n" +
				"mov v0, va1", 1);
			validVertex = vertexAssembly.agalcode;

			var validFragment:ByteArray = makeValidFragment();

			// 1. Valid shaders
			tryUpload("valid", validVertex, validFragment);

			// 2. Sampler config mismatch
			var fragmentAssembly:AGALMiniAssembler = new AGALMiniAssembler(false);
			fragmentAssembly.assemble(Context3DProgramType.FRAGMENT,
				"tex ft0, v0, fs0 <2d,repeat,linear,mipnone>\n" +
				"tex ft1, v0, fs0 <2d,repeat,linear,mipnearest>\n" +
				"add oc, ft0, ft1", 1);
			tryUpload("sampler_conflict", validVertex, fragmentAssembly.agalcode);

			// 3. Invalid header (bad magic byte)
			var badHeader:ByteArray = new ByteArray();
			badHeader.writeByte(0xFF); // wrong magic
			for (var i:int = 1; i < 31; i++) badHeader.writeByte(0);
			badHeader.position = 0;
			tryUpload("bad_header", validVertex, badHeader);

			// 4. Truncated bytecode (too short for header)
			var truncated:ByteArray = new ByteArray();
			truncated.writeByte(0xa0);
			truncated.writeByte(0x01);
			truncated.position = 0;
			tryUpload("truncated", validVertex, truncated);

			// 5. Empty bytecode
			var empty:ByteArray = new ByteArray();
			tryUpload("empty", validVertex, empty);

			// 6. Invalid version
			var badVersion:ByteArray = new ByteArray();
			badVersion.endian = "littleEndian";
			badVersion.writeByte(0xa0);
			badVersion.writeUnsignedInt(99); // bad version
			badVersion.writeByte(0xa1);
			badVersion.writeByte(0x01);
			badVersion.position = 0;
			tryUpload("bad_version", validVertex, badVersion);

			// 6b. Invalid shader type
			var badShaderType:ByteArray = new ByteArray();
			badShaderType.endian = "littleEndian";
			badShaderType.writeByte(0xa0);
			badShaderType.writeUnsignedInt(1); // valid version
			badShaderType.writeByte(0xa1);
			badShaderType.writeByte(0x02); // not vertex (0x00) or fragment (0x01)
			badShaderType.position = 0;
			tryUpload("bad_shader_type", validVertex, badShaderType);

			// 7. Invalid opcode (valid header + bogus instruction)
			var badOpcode:ByteArray = new ByteArray();
			badOpcode.endian = "littleEndian";
			badOpcode.writeByte(0xa0);
			badOpcode.writeUnsignedInt(1);
			badOpcode.writeByte(0xa1);
			badOpcode.writeByte(0x01);
			// 24-byte instruction with opcode 0xFF
			badOpcode.writeUnsignedInt(0xFF);
			for (var j:int = 0; j < 5; j++) badOpcode.writeUnsignedInt(0);
			badOpcode.position = 0;
			tryUpload("bad_opcode", validVertex, badOpcode);

			// 8. Bad source register type: reading from Output as source in fragment shader
			// "mov ft0, oc" - reading output register as source
			var badSourceOutput:ByteArray = new ByteArray();
			badSourceOutput.endian = "littleEndian";
			writeFragmentHeader(badSourceOutput);
			writeMov(badSourceOutput, REG_TEMPORARY, 0, REG_OUTPUT, 0);
			writeMov(badSourceOutput, REG_OUTPUT, 0, REG_CONSTANT, 0);
			badSourceOutput.position = 0;
			tryUpload("source_output", validVertex, badSourceOutput);

			// 9. Bad source register type: reading from Sampler as source
			// "mov ft0, fs0" - reading sampler register as source
			var badSourceSampler:ByteArray = new ByteArray();
			badSourceSampler.endian = "littleEndian";
			writeFragmentHeader(badSourceSampler);
			writeMov(badSourceSampler, REG_TEMPORARY, 0, REG_SAMPLER, 0);
			writeMov(badSourceSampler, REG_OUTPUT, 0, REG_CONSTANT, 0);
			badSourceSampler.position = 0;
			tryUpload("source_sampler", validVertex, badSourceSampler);

			// 10. Bad dest register type: writing to Constant
			// "mov fc0, ft0" - writing to constant register
			var badDestConstant:ByteArray = new ByteArray();
			badDestConstant.endian = "littleEndian";
			writeFragmentHeader(badDestConstant);
			writeMov(badDestConstant, REG_CONSTANT, 0, REG_CONSTANT, 0);
			badDestConstant.position = 0;
			tryUpload("dest_constant", validVertex, badDestConstant);

			// 11. Bad dest register type: writing to Attribute in vertex shader
			// "mov va0, vc0" - writing to attribute register
			var badDestAttr:ByteArray = new ByteArray();
			badDestAttr.endian = "littleEndian";
			writeVertexHeader(badDestAttr);
			writeMov(badDestAttr, REG_ATTRIBUTE, 0, REG_CONSTANT, 0);
			badDestAttr.position = 0;
			tryUpload("dest_attribute", badDestAttr, makeValidFragment());

			// 12. Bad dest register type: writing to Sampler
			// "mov fs0, fc0" - writing to sampler register
			var badDestSampler:ByteArray = new ByteArray();
			badDestSampler.endian = "littleEndian";
			writeFragmentHeader(badDestSampler);
			writeMov(badDestSampler, REG_SAMPLER, 0, REG_CONSTANT, 0);
			badDestSampler.position = 0;
			tryUpload("dest_sampler", validVertex, badDestSampler);

			// 12b. Bad dest register type: writing to FragmentRegister
			// "mov fd0, fc0" - writing to fragment register
			var badDestFragReg:ByteArray = new ByteArray();
			badDestFragReg.endian = "littleEndian";
			writeFragmentHeader(badDestFragReg);
			writeMov(badDestFragReg, REG_FRAGMENT, 0, REG_CONSTANT, 0);
			badDestFragReg.position = 0;
			tryUpload("dest_fragreg", validVertex, badDestFragReg);

			// 13. Indirect mode in fragment shader
			// "mov oc, ft0[fc0.x+0]" - indirect access in fragment shader
			var badIndirectFrag:ByteArray = new ByteArray();
			badIndirectFrag.endian = "littleEndian";
			writeFragmentHeader(badIndirectFrag);
			writeMov(badIndirectFrag, REG_OUTPUT, 0, REG_TEMPORARY, 0,
				true, REG_CONSTANT, 0, 0);
			badIndirectFrag.position = 0;
			tryUpload("indirect_frag", validVertex, badIndirectFrag);

			// 14. Indirect mode with non-Constant source in vertex shader
			// "mov op, vt0[va0.x+0]" - indirect on temporary register
			var badIndirectNonConst:ByteArray = new ByteArray();
			badIndirectNonConst.endian = "littleEndian";
			writeVertexHeader(badIndirectNonConst);
			writeMov(badIndirectNonConst, REG_OUTPUT, 0, REG_TEMPORARY, 0,
				true, REG_ATTRIBUTE, 0, 0);
			badIndirectNonConst.position = 0;
			tryUpload("indirect_non_const", badIndirectNonConst, makeValidFragment());

			// 15. Valid indirect mode: vc[va0.x+0] in vertex shader
			var validIndirect:ByteArray = new ByteArray();
			validIndirect.endian = "littleEndian";
			writeVertexHeader(validIndirect);
			writeMov(validIndirect, REG_OUTPUT, 0, REG_CONSTANT, 0,
				true, REG_ATTRIBUTE, 0, 0);
			validIndirect.position = 0;
			tryUpload("indirect_valid", validIndirect, makeValidFragment());

			// 16. Source register type FragmentRegister (type 6)
			var badSourceFragReg:ByteArray = new ByteArray();
			badSourceFragReg.endian = "littleEndian";
			writeFragmentHeader(badSourceFragReg);
			writeMov(badSourceFragReg, REG_TEMPORARY, 0, REG_FRAGMENT, 0);
			writeMov(badSourceFragReg, REG_OUTPUT, 0, REG_CONSTANT, 0);
			badSourceFragReg.position = 0;
			tryUpload("source_fragreg", validVertex, badSourceFragReg);
		}
	}
}
