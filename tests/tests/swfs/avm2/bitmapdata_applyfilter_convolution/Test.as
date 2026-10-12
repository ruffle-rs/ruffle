package {
	import flash.display.BitmapData;
	import flash.display.DisplayObjectContainer;
	import flash.display.MovieClip;
	import flash.filters.BitmapFilter;
	import flash.filters.ConvolutionFilter;
	import flash.geom.Point;
	import flash.geom.Rectangle;

	/**
	 * Numeric coverage for flash.filters.ConvolutionFilter.
	 *
	 * The first kernel is the one net.flashfilter's E01_raindrop water-ripple
	 * simulation applies every frame to propagate its wave height field:
	 * a 3x3 box blur with divisor 9 and bias 0.
	 */
	public class Test extends MovieClip {
		static const W: uint = 20;
		static const H: uint = 20;

		public function Test() {
			Test.run(this);
		}

		public static function run(root: DisplayObjectContainer): void {
			// A single bright pixel on a black background, so the spread of the
			// kernel is fully observable, plus a flat region and alpha coverage.
			var src: BitmapData = new BitmapData(W, H, true, 0xFF000000);
			src.setPixel32(10, 10, 0xFFFFFFFF);
			for (var y: uint = 14; y < 19; y++) {
				for (var x: uint = 14; x < 19; x++) {
					src.setPixel32(x, y, 0xFF808080);
				}
			}
			// A fully transparent pixel, to check that alpha is convolved too
			src.setPixel32(1, 1, 0x00000000);

			var box: ConvolutionFilter = new ConvolutionFilter(3, 3, [1, 1, 1, 1, 1, 1, 1, 1, 1], 9, 0, true, true, 128);
			trace("// source: red channel of the 3x3 around the white pixel at (10,10)");
			dumpChannels(src, 9, 9, 3, 3);

			var blurred: BitmapData = apply(src, box, new Rectangle(0, 0, W, H));
			trace("// 3x3 box blur (divisor 9): red channel, same region");
			dumpChannels(blurred, 9, 9, 3, 3);

			trace("// 3x3 box blur: red channel of the whole row y=10");
			var row: String = "";
			for (var x: uint = 0; x < W; x++) {
				row += (blurred.getPixel32(x, 10) >>> 16 & 0xFF) + " ";
			}
			trace(row);

			// 255/9 = 28.33 -> 28, and the 5x5 block of 128 stays 128
			trace("// blur of the white pixel spread: " + (blurred.getPixel32(10, 10) >>> 16 & 0xFF));
			trace("// blur of the flat 128 block: " + (blurred.getPixel32(16, 16) >>> 16 & 0xFF));

			// The transparent pixel at (1,1) keeps the source alpha (preserveAlpha)
			trace("// alpha at the transparent source pixel (1,1): " + (blurred.getPixel32(1, 1) >>> 24));
			// ...but its neighbours pick up a partial alpha from it
			trace("// alpha one pixel away (2,2): " + (blurred.getPixel32(2, 2) >>> 24));
			// preserveAlpha must not let transparent neighbours darken the colour,
			// so the surrounding white background keeps its 255 red channel
			trace("// red one pixel away (2,2): " + (blurred.getPixel32(2, 2) >>> 16 & 0xFF));

			// Bias is added after the division
			var biased: ConvolutionFilter = new ConvolutionFilter(3, 3, [1, 1, 1, 1, 1, 1, 1, 1, 1], 9, 10, true, true, 128);
			trace("// blur with bias 10, flat 128 block: " + (apply(src, biased, new Rectangle(0, 0, W, H)).getPixel32(16, 16) >>> 16 & 0xFF));

			// A zero divisor is treated as 1 by Flash
			var zeroDivisor: ConvolutionFilter = new ConvolutionFilter(3, 3, [1, 1, 1, 1, 1, 1, 1, 1, 1], 0, 0, true, true, 128);
			trace("// blur with divisor 0, flat 128 block: " + (apply(src, zeroDivisor, new Rectangle(0, 0, W, H)).getPixel32(16, 16) >>> 16 & 0xFF));

			// Edge detection: a flat region becomes black everywhere
			var edge: ConvolutionFilter = new ConvolutionFilter(3, 3, [0, -1, 0, -1, 4, -1, 0, -1, 0], 1, 0, true, true, 128);
			var edges: BitmapData = apply(src, edge, new Rectangle(0, 0, W, H));
			trace("// edge kernel: flat 128 block: " + (edges.getPixel32(16, 16) >>> 16 & 0xFF));
			trace("// edge kernel: the isolated white pixel: " + (edges.getPixel32(10, 10) >>> 16 & 0xFF));

			// Negative results clamp to zero, and the 1px transparent pixel
			// also produces an edge
			trace("// edge kernel: black background: " + (edges.getPixel32(5, 5) >>> 16 & 0xFF));

			// sourceRect handling: only the requested region is written, and
			// taps outside it use the default colour when clamp is false
			var unclamped: ConvolutionFilter = new ConvolutionFilter(3, 3, [1, 1, 1, 1, 1, 1, 1, 1, 1], 9, 0, false, true, 0xFF00FF00);
			var outside: BitmapData = apply(src, unclamped, new Rectangle(5, 5, 10, 10));
			trace("// unclamped, outside the sourceRect (0,0) rgb: " + hex(outside.getPixel32(0, 0)));
			trace("// unclamped, on the sourceRect corner (5,5) rgb: " + hex(outside.getPixel32(5, 5)));
			trace("// unclamped, below the sourceRect (5,15) rgb: " + hex(outside.getPixel32(5, 15)));

			// With clamp, the same taps reuse the closest pixel in the region
			var clamped: ConvolutionFilter = new ConvolutionFilter(3, 3, [1, 1, 1, 1, 1, 1, 1, 1, 1], 9, 0, true, true, 0xFF00FF00);
			var inside: BitmapData = apply(src, clamped, new Rectangle(5, 5, 10, 10));
			trace("// clamped, on the sourceRect corner (5,5) rgb: " + hex(inside.getPixel32(5, 5)));
			trace("// clamped, below the sourceRect (5,15) rgb: " + hex(inside.getPixel32(5, 15)));

			// Flash convolves 8-bit integer pixel values, so an average that
			// lands exactly on a whole number must stay there. An implementation
			// that accumulates normalised 0..1 texel values can land a hair
			// below the integer and truncate one short, so every pixel of a
			// pseudo-random pattern is checked against exact int arithmetic.
			var noise: BitmapData = new BitmapData(W, H, false, 0xFF000000);
			var seed: uint = 12345;
			for (var ny: uint = 0; ny < H; ny++) {
				for (var nx: uint = 0; nx < W; nx++) {
					seed = seed * 1664525 + 1013904223;
					noise.setPixel32(nx, ny, 0xFF000000 | ((seed >>> 16) & 0xFF));
				}
			}
			var blurredNoise: BitmapData = apply(noise, box, new Rectangle(0, 0, W, H));
			var wrong: int = 0;
			var example: String = "";
			for (ny = 0; ny < H; ny++) {
				for (nx = 0; nx < W; nx++) {
					var sum: int = 0;
					for (var dy: int = -1; dy <= 1; dy++) {
						for (var dx: int = -1; dx <= 1; dx++) {
							var sx: int = Math.min(Math.max(nx + dx, 0), W - 1);
							var sy: int = Math.min(Math.max(ny + dy, 0), H - 1);
							sum += noise.getPixel32(sx, sy) & 0xFF;
						}
					}
					var want: int = sum / 9;
					var got: int = blurredNoise.getPixel32(nx, ny) & 0xFF;
					if (got != want) {
						wrong++;
						if (example == "") {
							example = " first (" + nx + "," + ny + "): got " + got + " want " + want;
						}
					}
				}
			}
			trace("// box blur of a pseudo-random pattern, mismatches vs exact floor(sum/9): " + wrong + example);

			// Flash Player clamps matrixX/matrixY to 0..15 at the property
			// boundary: assigning 16/31/255 reads back as 15.
			var prop15: Array = [];
			for (var pi: int = 0; pi < 15; pi++) { prop15.push(1); }
			var prop16: Array = [];
			for (pi = 0; pi < 16; pi++) { prop16.push(1); }
			var prop31: Array = [];
			for (pi = 0; pi < 31 * 5; pi++) { prop31.push(1); }
			var prop255: Array = [];
			for (pi = 0; pi < 255; pi++) { prop255.push(1); }
			trace("// matrixX after set 15: " + new ConvolutionFilter(15, 1, prop15).matrixX);
			trace("// matrixX after set 16: " + new ConvolutionFilter(16, 1, prop16).matrixX);
			trace("// matrixX after set 31: " + new ConvolutionFilter(31, 5, prop31).matrixX);
			trace("// matrixX after set 255: " + new ConvolutionFilter(255, 1, prop255).matrixX);
			// A flat value-1 region reports the number of active taps: 16
			// declared weights become 15.
			var flatOne: BitmapData = new BitmapData(40, 4, false, 0xFF010101);
			var ones16: ConvolutionFilter = new ConvolutionFilter(16, 1, prop16, 1, 0, true, true);
			var ones16Dst: BitmapData = new BitmapData(40, 4, true, 0);
			ones16Dst.applyFilter(flatOne, new Rectangle(0, 0, 40, 4), new Point(0, 0), ones16);
			trace("// 16x1 all-ones on flat value 1: " + (ones16Dst.getPixel32(20, 2) & 0xFF));
			// Weights 0..14 sum to 105: the FIRST 15 entries survive
			// (top-left sub-matrix), entry 15 is dropped.
			var ramp16: Array = [];
			for (pi = 0; pi < 16; pi++) { ramp16.push(pi); }
			var ramp16Filter: ConvolutionFilter = new ConvolutionFilter(16, 1, ramp16, 1, 0, true, true);
			var ramp16Dst: BitmapData = new BitmapData(40, 4, true, 0);
			ramp16Dst.applyFilter(flatOne, new Rectangle(0, 0, 40, 4), new Point(0, 0), ramp16Filter);
			trace("// 16x1 weights 0..15 on flat value 1: " + (ramp16Dst.getPixel32(20, 2) & 0xFF));
			// The surviving sub-matrix keeps the original anchor convention:
			// entry 0 lands at offset -7, and entry 15 is not sampled.
			var oneLine: BitmapData = new BitmapData(40, 4, false, 0xFF000000);
			oneLine.setPixel32(10, 2, 0xFFFFFFFF);
			var w0: Array = [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
			var w0Filter: ConvolutionFilter = new ConvolutionFilter(16, 1, w0, 1, 0, true, true);
			var w0Dst: BitmapData = new BitmapData(40, 4, true, 0);
			w0Dst.applyFilter(oneLine, new Rectangle(0, 0, 40, 4), new Point(0, 0), w0Filter);
			trace("// 16x1 w[0]=1, dest x=17: " + (w0Dst.getPixel32(17, 2) & 0xFF) + " dest x=18: " + (w0Dst.getPixel32(18, 2) & 0xFF));
			var w15: Array = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];
			var w15Filter: ConvolutionFilter = new ConvolutionFilter(16, 1, w15, 1, 0, true, true);
			var w15Dst: BitmapData = new BitmapData(40, 4, true, 0);
			w15Dst.applyFilter(oneLine, new Rectangle(0, 0, 40, 4), new Point(0, 0), w15Filter);
			trace("// 16x1 w[15]=1, dest x=3: " + (w15Dst.getPixel32(3, 2) & 0xFF) + " dest x=2: " + (w15Dst.getPixel32(2, 2) & 0xFF));

			// Zero-weight packing: only non-zero entries are uploaded. A
			// 255x255 kernel declared with 65,025 entries is clamped to its
			// top-left 15x15, where only entry 0 survives; the opposite
			// corner weight past the sub-matrix is dropped.
			var sparseDims: int = 255;
			var sparseMatrix: Array = [];
			for (var si: int = 0; si < sparseDims * sparseDims; si++) {
				sparseMatrix.push(0);
			}
			sparseMatrix[0] = 1;
			sparseMatrix[sparseDims * sparseDims - 1] = 1;
			var sparse255: ConvolutionFilter = new ConvolutionFilter(255, 255, sparseMatrix, 2, 0, true, true);
			var sparseSrc: BitmapData = new BitmapData(W, H, false, 0xFF000000);
			sparseSrc.setPixel32(0, 0, 0xFF202020);
			sparseSrc.setPixel32(W - 1, H - 1, 0xFF606060);
			var sparseDst: BitmapData = apply(sparseSrc, sparse255, new Rectangle(0, 0, W, H));
			// Effective kernel: one weight-1 tap at offset (-7,-7), divisor 2.
			// At (0,0) it clamps to (0,0): 32 / 2 = 16. Elsewhere it samples
			// black, so the result is 0.
			trace("// clamped 255x255 sparse kernel, (0,0): " + (sparseDst.getPixel32(0, 0) & 0xFF));
			trace("// clamped 255x255 sparse kernel, (10,10): " + (sparseDst.getPixel32(10, 10) & 0xFF));
			trace("// clamped 255x255 sparse kernel, (19,19): " + (sparseDst.getPixel32(19, 19) & 0xFF));

			// A 31x31-declared sparse kernel renders exactly like a 15x15:
			// five weighted taps inside the top-left 15x15, at the four
			// sub-matrix corners and its center. Every output pixel is
			// checked against an integer reference applying the same clamp.
			var sparse31Size: int = 31;
			var sparse31: Array = [];
			for (si = 0; si < sparse31Size * sparse31Size; si++) {
				sparse31.push(0);
			}
			// Positions are flat indices into the clamped 15x15 matrix:
			// Flash reads the first 225 array entries with the clamped
			// stride, not the top-left corner of the 31-wide layout.
			sparse31[0] = 1;                                  // ( 0, 0) off (-7,-7)
			sparse31[14] = 2;                                 // ( 0,14) off ( 7,-7)
			sparse31[210] = 3;                                // (14, 0) off (-7, 7)
			sparse31[224] = 4;                                // (14,14) off ( 7, 7)
			sparse31[112] = 10;                               // ( 7, 7) center
			var f31: ConvolutionFilter = new ConvolutionFilter(sparse31Size, sparse31Size, sparse31, 20, 0, true, true);
			var mid: BitmapData = new BitmapData(40, 40, false, 0xFF000000);
			mid.setPixel32(5, 5, 0xFF0A0A0A);   // 10
			mid.setPixel32(35, 5, 0xFF141414);  // 20
			mid.setPixel32(5, 35, 0xFF1E1E1E);  // 30
			mid.setPixel32(35, 35, 0xFF282828); // 40
			mid.setPixel32(20, 20, 0xFF323232); // 50
			var sparse31Dst: BitmapData = new BitmapData(40, 40, true, 0x00000000);
			sparse31Dst.applyFilter(mid, new Rectangle(0, 0, 40, 40), new Point(0, 0), f31);
			trace("// 31x31-declared sparse kernel, center (20,20): " + (sparse31Dst.getPixel32(20, 20) & 0xFF));
			trace("// 31x31-declared sparse kernel, (5,5): " + (sparse31Dst.getPixel32(5, 5) & 0xFF));
			trace("// 31x31-declared sparse kernel, (35,5): " + (sparse31Dst.getPixel32(35, 5) & 0xFF));
			trace("// 31x31-declared sparse kernel, (5,35): " + (sparse31Dst.getPixel32(5, 35) & 0xFF));
			trace("// 31x31-declared sparse kernel, (35,35): " + (sparse31Dst.getPixel32(35, 35) & 0xFF));
			var sparse31Wrong: int = 0;
			var sparse31Example: String = "";
			for (var cy: int = 0; cy < 40; cy++) {
				for (var cx: int = 0; cx < 40; cx++) {
					var want31: int = sparseReference(mid, sparse31, sparse31Size, cx, cy);
					var got31: int = sparse31Dst.getPixel32(cx, cy) & 0xFF;
					if (got31 != want31) {
						sparse31Wrong++;
						if (sparse31Example == "") {
							sparse31Example = " first (" + cx + "," + cy + "): got " + got31 + " want " + want31;
						}
					}
				}
			}
			trace("// 31x31-declared sparse kernel, mismatches vs clamped int reference: " + sparse31Wrong + sparse31Example);

			// A 20x20-declared dense box kernel renders as 15x15 (225 taps)
			// while keeping the user divisor (400). Over a centered 20x20
			// white square every output pixel is checked.
			var denseSize: int = 40;
			var denseSrc: BitmapData = new BitmapData(denseSize, denseSize, false, 0xFF000000);
			for (var dy: int = 0; dy < denseSize; dy++) {
				for (var dx2: int = 0; dx2 < denseSize; dx2++) {
					if (dx2 >= 10 && dx2 <= 29 && dy >= 10 && dy <= 29) {
						denseSrc.setPixel32(dx2, dy, 0xFFFFFFFF);
					}
				}
			}
			var denseMatrix: Array = [];
			for (si = 0; si < 400; si++) {
				denseMatrix.push(1);
			}
			var dense20: ConvolutionFilter = new ConvolutionFilter(20, 20, denseMatrix, 400, 0, true, true);
			var denseDst: BitmapData = new BitmapData(denseSize, denseSize, true, 0x00000000);
			denseDst.applyFilter(denseSrc, new Rectangle(0, 0, denseSize, denseSize), new Point(0, 0), dense20);
			var denseWrong: int = 0;
			var denseExample: String = "";
			for (dy = 0; dy < denseSize; dy++) {
				for (dx2 = 0; dx2 < denseSize; dx2++) {
					var hits: int = 0;
					// Effective 15x15 kernel, anchor at (7,7).
					for (var ky: int = 0; ky < 15; ky++) {
						for (var kx: int = 0; kx < 15; kx++) {
							var sx2: int = dx2 + kx - 7;
							var sy2: int = dy + ky - 7;
							if (sx2 < 0) {
								sx2 = 0;
							} else if (sx2 > denseSize - 1) {
								sx2 = denseSize - 1;
							}
							if (sy2 < 0) {
								sy2 = 0;
							} else if (sy2 > denseSize - 1) {
								sy2 = denseSize - 1;
							}
							if (sx2 >= 10 && sx2 <= 29 && sy2 >= 10 && sy2 <= 29) {
								hits++;
							}
						}
					}
					// The user-supplied divisor is preserved (400), not
					// renormalised to the 225-tap effective sum.
					var denseWant: int = hits * 255 / 400;
					var denseGot: int = denseDst.getPixel32(dx2, dy) & 0xFF;
					if (denseGot != denseWant) {
						denseWrong++;
						if (denseExample == "") {
							denseExample = " first (" + dx2 + "," + dy + "): got " + denseGot + " want " + denseWant;
						}
					}
				}
			}
			trace("// 20x20-declared dense box (effective 15x15), mismatches vs exact floor(hits*255/400): " + denseWrong + denseExample);
			trace("// 20x20-declared dense box, center (20,20): " + (denseDst.getPixel32(20, 20) & 0xFF));
			trace("// 20x20-declared dense box, ramp (10,10): " + (denseDst.getPixel32(10, 10) & 0xFF));
			trace("// 20x20-declared dense box, ramp (15,15): " + (denseDst.getPixel32(15, 15) & 0xFF));
			trace("// 20x20-declared dense box, corner (0,0): " + (denseDst.getPixel32(0, 0) & 0xFF));
		}

		static function sparseReference(data: BitmapData, matrix: Array, declaredSize: int, x: int, y: int): int {
			// Flash clamps the dimensions first, then reads the matrix as the
			// flat first size*size entries with the clamped stride.
			var size: int = Math.min(declaredSize, 15);
			var center: int = size / 2;
			var sum: int = 0;
			for (var row: int = 0; row < size; row++) {
				for (var col: int = 0; col < size; col++) {
					var weight: int = matrix[row * size + col];
					if (weight == 0) {
						continue;
					}
					var sx: int = x + col - center;
					var sy: int = y + row - center;
					if (sx < 0) {
						sx = 0;
					} else if (sx > data.width - 1) {
						sx = data.width - 1;
					}
					if (sy < 0) {
						sy = 0;
					} else if (sy > data.height - 1) {
						sy = data.height - 1;
					}
					sum += (data.getPixel32(sx, sy) & 0xFF) * weight;
				}
			}
			return sum / 20;
		}

		static function hex(pixel: uint): String {
			return "0x" + pixel.toString(16);
		}

		static function dumpChannels(data: BitmapData, x0: uint, y0: uint, w: uint, h: uint): void {
			for (var y: uint = y0; y < y0 + h; y++) {
				var row: String = "";
				for (var x: uint = x0; x < x0 + w; x++) {
					row += (data.getPixel32(x, y) >>> 16 & 0xFF) + " ";
				}
				trace(row);
			}
		}

		static function apply(src: BitmapData, filter: BitmapFilter, rect: Rectangle): BitmapData {
			var dst: BitmapData = new BitmapData(W, H, true, 0x00000000);
			dst.applyFilter(src, rect, new Point(0, 0), filter);
			return dst;
		}
	}
}
