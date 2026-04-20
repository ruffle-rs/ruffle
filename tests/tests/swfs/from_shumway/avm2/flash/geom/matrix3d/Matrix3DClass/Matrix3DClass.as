/*
   Compiled with:
   java -jar utils/asc.jar -import playerglobal.abc -swf Matrix3DTest,100,100,10 test/swfs/flash_geom_Matrix3D.as
*/

﻿package  {
	
	import flash.display.MovieClip;
	
	
	public class Matrix3DTest extends MovieClip {
		public function Matrix3DTest() {
			var test: NativeMatrix3DUnitTest = new NativeMatrix3DUnitTest();
			test.createData();

			test.constructIdentity();
			test.construct();
			test.clone();
			test.newScale();
			test.newTranslation();
			test.newRotationBasics();
			test.newRotation();
			test.append();
			test.appendScale();
			test.appendTranslation();
			test.appendRotation();
			test.prepend_fixed();
			test.prependScale();
			test.prependTranslation();
			test.prependRotation();
			test.determinant();
			test.invert();
			test.deltaTransformVector();
			test.transformVector_fixed();
			test.transformVectors();
			test.getPosition();
			test.setPosition();
			test.getRawData16();
			test.setRawData16();
		}
	}
	
}

class Assert {
	public static function assertEquals(message: String, actual: Object, expected: Object)
	{
		// Shumway cannot do exact comparison of the objects
		if (typeof actual == typeof expected && String(actual) == String(expected)) {
			trace('SUCCESS | ' + message);
		} else {
			trace('FAILED | ' + message + ' | Expected: ' + expected + '; actual: ' + actual);
		}
	}

	public static function assertTrue(message: String, cond: Boolean)
	{
		assertEquals(message, cond, true);
	}

	public static function assertFalse(message: String, cond: Boolean)
	{
		assertEquals(message, cond, false);
	}
}

// Original at https://raw.github.com/richardlord/Coral/master/native/test/NativeMatrix3DUnitTest.as
/*
CORAL 3D-MATHEMATICS
....................

Author: Richard Lord
Copyright (c) Richard Lord 2008-2011
http://www.richardlord.net/


Licence Agreement

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
THE SOFTWARE.
*/
	import flash.geom.Matrix3D;
	import flash.geom.Vector3D;

	class NativeMatrix3DUnitTest
	{
		private var e : Number = 0.001;
		
		private var matrix1 : Matrix3D;
		private var determinant1 : Number;
		private var inverse1 : Matrix3D;
		
		private var matrix2 : Matrix3D;
		private var determinant2 : Number;
		private var inverse2 : Matrix3D;
		
		private var pre1 : Matrix3D;
		private var post1 : Matrix3D;
		private var result1 : Matrix3D;
		
		private var pre2 : Matrix3D;
		private var post2 : Matrix3D;
		private var result2 : Matrix3D;
		
		private var pre3 : Matrix3D;
		private var post3 : Matrix3D;
		private var result3 : Matrix3D;
		
		private var transform1 : Matrix3D;
		private var point1 : Vector3D;
		private var vector1 : Vector3D;
		private var transformedPoint1 : Vector3D;
		private var transformedVector1 : Vector3D;
		
		private var transform2 : Matrix3D;
		private var point2 : Vector3D;
		private var vector2 : Vector3D;
		private var transformedPoint2 : Vector3D;
		private var transformedVector2 : Vector3D;
		
		private var transform3 : Matrix3D;
		private var point3 : Vector3D;
		private var transformedPoint3 : Vector3D;
		
		[Before]
		public function createData() : void
		{
			matrix1 = new Matrix3D( Vector.<Number>( [ 1, 2, 1, 2, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 3 ] ) );
			determinant1 = 10;
			inverse1 = new Matrix3D( Vector.<Number>( [ -0.8, 0.5, 0.3, -0.1, -0.2, 0.5, -0.3, 0.1, -1.4, 0.5, -0.1, 0.7, 1.8, -1, 0.2, -0.4 ] ) );
			
			matrix2 = new Matrix3D( Vector.<Number>( [ 2, 1, -1, 0, -1, 3, -2, 0, 3, 2, -2, 0, 0, 1, 1, 1 ] ) );
			determinant2 = -1;
			inverse2 = new Matrix3D( Vector.<Number>( [ 2, 0, -1, 0, 8, 1, -5, 0, 11, 1, -7, 0, -19, -2, 12, 1 ] ) );
			
			pre1 = new Matrix3D( Vector.<Number>( [ 1, 2, 1, 4, 2, 3, 2, 1, 3, 3, 4, 2, 4, 2, 4, 1 ] ) );
			post1 = new Matrix3D( Vector.<Number>( [ 3, 2, 4, 2, 1, 1, 1, 3, 2, 2, 2, 4, 3, 4, 1, 4 ] ) );
			result1 = new Matrix3D( Vector.<Number>( [ 27, 28, 31, 24, 18, 14, 19, 10, 28, 24, 30, 18, 30, 29, 31, 22 ] ) );
			
			pre2 = new Matrix3D( Vector.<Number>( [ 2, 1, 4, 1, 3, 5, 2, 2, 2, 4, 4, 1, 3, 3, 5, 1 ] ) );
			post2 = new Matrix3D( Vector.<Number>( [ 3, 2, 4, 3, 2, 1, 2, 4, 4, 2, 3, 2, 3, 3, 1, 1 ] ) );
			result2 = new Matrix3D( Vector.<Number>( [ 29, 38, 47, 14, 23, 27, 38, 10, 26, 32, 42, 13, 20, 25, 27, 11 ] ) );

			pre3 = new Matrix3D( Vector.<Number>( [ 2, 1, 4, 0, 3, 5, 2, 0, 2, 4, 4, 0, 3, 3, 5, 1 ] ) );
			post3 = new Matrix3D( Vector.<Number>( [ 3, 2, 4, 0, 2, 1, 2, 0, 4, 2, 3, 0, 3, 3, 1, 1 ] ) );
			result3 = new Matrix3D( Vector.<Number>( [ 20, 29, 32, 0, 11, 15, 18, 0, 20, 26, 32, 0, 20, 25, 27, 1 ] ) );

			transform1 = new Matrix3D( Vector.<Number>( [ 1, 4, 2, 0, 2, 3, 4, 0, 4, 2, 3, 0, 3, 1, 2, 1 ] ) );
			point1 = new Vector3D( 2, 3, 1 );
			transformedPoint1 = new Vector3D( 15, 20, 21 );
			vector1 = new Vector3D( 2, 3, 1 );
			transformedVector1 = new Vector3D( 12, 19, 19 );
			
			transform2 = new Matrix3D( Vector.<Number>( [ 4, 1, 3, 3, 2, -2, 2, 2, 2, 4, 1, 2, 3, -1, 1, 4 ] ) );
			point2 = new Vector3D( -3, 2, 2 );
			transformedPoint2 = new Vector3D( -1, 0, -2 );
			transformedPoint2.w = 3;
			vector2 = new Vector3D( -3, 2, 2 );
			transformedVector2 = new Vector3D( -4, 1, -3 );
			transformedVector2.w = -1;
			
			transform3 = new Matrix3D( Vector.<Number>( [ 1, -2, 4, 2, 3, 1, 1, -1, 2, -1, 3, 2, 1, 3, 3, 4 ] ) );
			point3 = new Vector3D( 2, -1, 3 );
			point3.w = 2;
			transformedPoint3 = new Vector3D( 7, -2, 22 );
			transformedPoint3.w = 19;
		}
		
		[Test]
		public function constructIdentity() : void
		{
			Assert.assertTrue( "Identity matrix correct", equalMatrices( new Matrix3D(), new Matrix3D( Vector.<Number>( [ 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1 ] ) ) ) );
			var random : Matrix3D = randomMatrix();
			var m:Matrix3D = random.clone();
			m.append( new Matrix3D() );
			Assert.assertTrue( "Identity matrix has no effect", equalMatrices( m, random ) );
			m = random.clone();
			m.prepend( new Matrix3D() );
			Assert.assertTrue( "Identity matrix has no effect", equalMatrices( m, random ) );
		}

		[Test]
		public function construct() : void
		{
			var a : Vector.<Number> = randomVectorNumber( 16 );
			var matrix : Matrix3D = new Matrix3D( a );
			var r : Vector.<Number> = matrix.rawData;
			Assert.assertTrue( "Construct correct", nearEqualVectorNums( r, a, e ) );
		}

		[Test]
		public function clone() : void
		{
			var m1 : Matrix3D = randomMatrix();
			var m2 : Matrix3D = m1.clone();
			Assert.assertTrue( "Clone equals original", equalMatrices( m1, m2 ) );
			Assert.assertFalse( "Cloned matrices are not the same", m1 == m2 );
		}

		[Test] // Sometimes fails: rounding errors exceed e
		public function newScale() : void
		{
			var x : Number = randomNumber();
			var y : Number = randomNumber();
			var z : Number = randomNumber();
			var m : Matrix3D = new Matrix3D();
			m.appendScale( x, y, z );
			
			var v : Vector3D = randomVector();
			var w : Vector3D = m.deltaTransformVector( v );
			var rv : Vector3D = new Vector3D( v.x * x, v.y * y, v.z * z, 0 );
			Assert.assertTrue( "Scale vector correct", w.nearEquals( rv, e ) );
			
			var p : Vector3D = randomPoint();
			var q : Vector3D = m.transformVector( p );
			var rp : Vector3D = new Vector3D( p.x * x, p.y * y, p.z * z, 1 );
			Assert.assertTrue( "Scale point correct", q.nearEquals( rp, e ) );
		}

		[Test]
		public function newTranslation() : void
		{
			var x : Number = randomNumber();
			var y : Number = randomNumber();
			var z : Number = randomNumber();
			var m : Matrix3D = new Matrix3D();
			m.appendTranslation( x, y, z );
			
			var v : Vector3D = randomVector();
			var w : Vector3D = m.deltaTransformVector( v );
			Assert.assertTrue( "Scale vector correct", w.nearEquals( v, e ) );
			
			var p : Vector3D = randomPoint();
			var q : Vector3D = m.transformVector( p );
			var rp : Vector3D = new Vector3D( p.x + x, p.y + y, p.z + z );
			Assert.assertTrue( "Scale point correct", q.nearEquals( rp, e ) );
		}

		[Test] // Fails: assumes axis is unit length
		public function newRotationBasics() : void
		{
			var deg : Number = randomNumber();
			var v : Vector3D = randomVector();
			var p : Vector3D = randomPoint();
			var m : Matrix3D = new Matrix3D();
			m.appendRotation( deg, v, p );
			
			var w : Vector3D = m.deltaTransformVector( v );
			Assert.assertTrue( "New rotation doesn't transform own axis", w.nearEquals( v, e ) );
			
			var q : Vector3D = m.transformVector( p );
			Assert.assertTrue( "New rotation doesn't transform point on axis", q.nearEquals( p, e ) );
			p.incrementBy( v );
			q = m.transformVector( p );
			Assert.assertTrue( "New rotation doesn't transform point on axis", q.nearEquals( p, e ) );
			
			v.normalize();
			var m2 : Matrix3D = new Matrix3D();
			m2.appendRotation( deg, v, p );
			Assert.assertTrue( "New rotation axis length is irrelevant", nearEqualMatrices( m, m2, e ) );
			
			p = randomPoint();
			m = new Matrix3D();
			m.appendRotation( 360, v, p );
			q = m.transformVector( p );
			Assert.assertTrue( "New rotation 360 degrees doesn't transform point", q.nearEquals( p, e ) );
		}

		[Test]
		public function newRotation() : void
		{
			var p : Vector3D = randomPoint();
			var m : Matrix3D = new Matrix3D();
			var u : Vector3D = new Vector3D( 1, 1, 1 );
			u.normalize();
			m.appendRotation( 120, u, p );
			
			var v : Vector3D = m.deltaTransformVector( Vector3D.X_AXIS );
			Assert.assertTrue( "New rotation transform on x axis", v.nearEquals( Vector3D.Y_AXIS, e ) );
			
			v = m.deltaTransformVector( Vector3D.Y_AXIS );
			Assert.assertTrue( "New rotation transform on y axis", v.nearEquals( Vector3D.Z_AXIS, e ) );
			
			v = m.deltaTransformVector( Vector3D.Z_AXIS );
			Assert.assertTrue( "New rotation transform on z axis", v.nearEquals( Vector3D.X_AXIS, e ) );
			
			var q : Vector3D = m.transformVector( p.add( Vector3D.X_AXIS ) );
			Assert.assertTrue( "New rotation transform on p + x axis", q.nearEquals( p.add( Vector3D.Y_AXIS ), e ) );
			
			q = m.transformVector( p.add( Vector3D.Y_AXIS ) );
			Assert.assertTrue( "New rotation transform on p + y axis", q.nearEquals( p.add( Vector3D.Z_AXIS ), e ) );
			
			q = m.transformVector( p.add( Vector3D.Z_AXIS ) );
			Assert.assertTrue( "New rotation transform on p + z axis", q.nearEquals( p.add( Vector3D.X_AXIS ), e ) );
		}

		[Test]
		public function append() : void
		{
			post1.append( pre1 );
			Assert.assertTrue( "Append 1 success", equalMatrices( post1, result1 ) );
			post2.append( pre2 );
			Assert.assertTrue( "Append 2 success", equalMatrices( post2, result2 ) );
		}

		[Test]
		public function appendScale() : void
		{
			var x : Number = randomNumber();
			var y : Number = randomNumber();
			var z : Number = randomNumber();

			var m1 : Matrix3D = randomMatrix();
			var m2 : Matrix3D = m1.clone();
			var m3 : Matrix3D = new Matrix3D();
			m3.appendScale( x, y, z );
			m1.append( m3 );
			m2.appendScale( x, y, z );
			Assert.assertTrue( "Append scale success", nearEqualMatrices( m1, m2, e ) );
		}

		[Test] // Fails: assumes bottom row of matrix is ( 0 0 0 1 )
		public function appendTranslation() : void
		{
			var x : Number = randomNumber();
			var y : Number = randomNumber();
			var z : Number = randomNumber();

			var m1 : Matrix3D = randomMatrix_fixed();
			var m2 : Matrix3D = m1.clone();
			var m3 : Matrix3D = new Matrix3D();
			m3.appendTranslation( x, y, z );
			m1.append( m3 );
			m2.appendTranslation( x, y, z );
			Assert.assertTrue( "Append translation success", nearEqualMatrices( m1, m2, e ) );
		}

		[Test]
		public function appendRotation() : void
		{
			var deg : Number = randomNumber();
			var v : Vector3D = randomVector();
			var p : Vector3D = randomPoint();

			var m1 : Matrix3D = randomMatrix();
			var m2 : Matrix3D = m1.clone();
			var m3 : Matrix3D = new Matrix3D();
			m3.appendRotation( deg, v, p );
			m1.append( m3 );
			m2.appendRotation( deg, v, p );
			Assert.assertTrue( "Append rotation success", nearEqualMatrices( m1, m2, e ) );
		}

		[Test]
		public function prepend() : void
		{
			pre1.prepend( post1 );
			Assert.assertTrue( "Prepend 1 success", nearEqualMatrices( pre1, result1, e ) );
			pre2.prepend( post2 );
			Assert.assertTrue( "Prepend 2 success", nearEqualMatrices( pre2, result2, e ) );
		}

		public function prepend_fixed() : void
		{
			pre3.prepend( post3 );
			Assert.assertTrue( "Prepend 3 success", nearEqualMatrices( pre3, result3, e ) );
		}

		[Test]
		public function prependScale() : void
		{
			var x : Number = randomNumber();
			var y : Number = randomNumber();
			var z : Number = randomNumber();

			var m1 : Matrix3D = randomMatrix();
			var m2 : Matrix3D = m1.clone();
			var m3 : Matrix3D = new Matrix3D();
			m3.appendScale( x, y, z );
			m1.prepend( m3 );
			m2.prependScale( x, y, z );
			Assert.assertTrue( "Prepend scale success", nearEqualMatrices( m1, m2, e ) );
		}

		[Test]
		public function prependTranslation() : void
		{
			var x : Number = randomNumber();
			var y : Number = randomNumber();
			var z : Number = randomNumber();

			var m1 : Matrix3D = randomMatrix();
			var m2 : Matrix3D = m1.clone();
			var m3 : Matrix3D = new Matrix3D();
			m3.appendTranslation( x, y, z );
			m1.prepend( m3 );
			m2.prependTranslation( x, y, z );
			Assert.assertTrue( "Prepend translation success", nearEqualMatrices( m1, m2, e ) );
		}

		[Test] // Sometimes fails: rounding errors exceed e
		public function prependRotation() : void
		{
			var deg : Number = randomNumber();
			var v : Vector3D = randomVector();
			var p : Vector3D = randomPoint();

			var m1 : Matrix3D = randomMatrix();
			var m2 : Matrix3D = m1.clone();
			var m3 : Matrix3D = new Matrix3D();
			m3.appendRotation( deg, v, p );
			m1.prepend( m3 );
			m2.prependRotation( deg, v, p );
			Assert.assertTrue( "Prepend rotation success", nearEqualMatrices( m1, m2, e ) );
		}

		[Test] // Fails: Just gets it wrong, for all matrices not just these two
		public function determinant() : void
		{
			Assert.assertEquals( "Matrix 1 determinant correct", matrix1.determinant, determinant1 );
			Assert.assertEquals( "Matrix 2 determinant correct", matrix2.determinant, determinant2 );
		}

		[Test]
		public function invert() : void
		{
			var m1 : Matrix3D = randomMatrix();
			var m2 : Matrix3D = m1.clone();
			m2.invert();
			m1.append( m2 );
			Assert.assertTrue( "Random inverse correct", nearEqualMatrices( m1, new Matrix3D(), e ) );

			matrix1.invert();
			Assert.assertTrue( "Matrix 1 inverse correct", nearEqualMatrices( matrix1, inverse1, e ) );
			matrix2.invert();
			Assert.assertTrue( "Matrix 2 inverse correct", nearEqualMatrices( matrix2, inverse2, e ) );
		}

		[Test]
		public function deltaTransformVector() : void
		{
			var v : Vector3D = transform1.deltaTransformVector( vector1 );
			Assert.assertTrue( "Vector transform one correct", v.nearEquals( transformedVector1, e ) );
			v = transform2.deltaTransformVector( vector2 );
			Assert.assertTrue( "Vector transform two correct", v.nearEquals( transformedVector2, e ) );
		}

		[Test] // Fails: ignores w in source
		public function transformVector() : void
		{
			var p : Vector3D = transform1.transformVector( point1 );
			Assert.assertTrue( "Point transform one correct", p.nearEquals( transformedPoint1, e ) );
			p = transform2.transformVector( point2 );
			Assert.assertTrue( "Point transform two correct", p.nearEquals( transformedPoint2, e ) );
			p = transform3.transformVector( point3 );
			Assert.assertTrue( "Point transform three correct", p.nearEquals( transformedPoint3, e ) );
		}

		public function transformVector_fixed() : void
		{
			var p : Vector3D = transform1.transformVector( point1 );
			Assert.assertTrue( "Point transform one correct", p.nearEquals( transformedPoint1, e ) );
			p = transform2.transformVector( point2 );
			Assert.assertTrue( "Point transform two correct", p.nearEquals( transformedPoint2, e ) );
		}

		[Test]
		public function transformVectors() : void
		{
			var m : Matrix3D = randomMatrix();
			var v : Vector.<Number> = new Vector.<Number>();
			var p1 : Vector3D = randomPoint();
			var p2 : Vector3D = randomPoint();
			var p3 : Vector3D = randomPoint();
			v.push( p1.x, p1.y, p1.z, p2.x, p2.y, p2.z, p3.x, p3.y, p3.z );
			var t : Vector.<Number> = new Vector.<Number>();
			var t1 : Vector3D = m.transformVector( p1 );
			var t2 : Vector3D = m.transformVector( p2 );
			var t3 : Vector3D = m.transformVector( p3 );
			t.push( t1.x, t1.y, t1.z, t2.x, t2.y, t2.z, t3.x, t3.y, t3.z );
			var r : Vector.<Number> = new Vector.<Number>();
			m.transformVectors( v, r );
			for( var i : uint = 0; i < 9; ++i )
			{
				Assert.assertTrue( "Points transform " + i, nearNumbers(r[i], t[i], e) );
			}
		}

		[Test]
		public function getPosition() : void
		{
			var m : Matrix3D = randomMatrix();
			var p : Vector3D = m.position;
			var r : Vector.<Number> = m.rawData;
			Assert.assertTrue( "Position x get correct", nearNumbers( p.x, r[12], e ) );
			Assert.assertTrue( "Position y get correct", nearNumbers( p.y, r[13], e ) );
			Assert.assertTrue( "Position z get correct", nearNumbers( p.z, r[14], e ) );
		}

		[Test]
		public function setPosition() : void
		{
			var m : Matrix3D = randomMatrix();
			var p : Vector3D = randomPoint();
			m.position = p;
			var r : Vector.<Number> = m.rawData;
			Assert.assertTrue( "Position x set correct", nearNumbers( p.x, r[12], e ) );
			Assert.assertTrue( "Position y set correct", nearNumbers( p.y, r[13], e ) );
			Assert.assertTrue( "Position z set correct", nearNumbers( p.z, r[14], e ) );
		}

		[Test]
		public function getRawData16() : void
		{
			var a : Vector.<Number> = randomVectorNumber( 16 );
			var matrix : Matrix3D = new Matrix3D( a );
			var rawData : Vector.<Number> = matrix.rawData;
			Assert.assertTrue( "Get raw data correct", nearEqualVectorNums( rawData, a, e ) );
		}

		[Test]
		public function setRawData16() : void
		{
			var a : Vector.<Number> = randomVectorNumber( 16 );
			var matrix : Matrix3D = randomMatrix();
			matrix.rawData = a;
			var correct : Matrix3D = new Matrix3D( a );
			Assert.assertTrue( "Set raw data with 12 parameters", nearEqualMatrices( matrix, correct, e ) );
		}

		private function randomNumber() : Number
		{
			return Math.random() * 200 - 100;
		}

		private function randomVector() : Vector3D
		{
			return new Vector3D( randomNumber(), randomNumber(), randomNumber(), 0 );
		}

		private function randomPoint() : Vector3D
		{
			return new Vector3D( randomNumber(), randomNumber(), randomNumber(), 1 );
		}

		private function randomMatrix() : Matrix3D
		{
			return new Matrix3D( Vector.<Number>( [
				randomNumber(), randomNumber(), randomNumber(), randomNumber(),
				randomNumber(), randomNumber(), randomNumber(), randomNumber(),
				randomNumber(), randomNumber(), randomNumber(), randomNumber(),
				randomNumber(), randomNumber(), randomNumber(), randomNumber()
			] ) );
		}

		private function randomMatrix_fixed() : Matrix3D
		{
			return new Matrix3D( Vector.<Number>( [
				randomNumber(), randomNumber(), randomNumber(), 0,
				randomNumber(), randomNumber(), randomNumber(), 0,
				randomNumber(), randomNumber(), randomNumber(), 0,
				randomNumber(), randomNumber(), randomNumber(), 1
			] ) );
		}

		private function randomVectorNumber( length : uint ) : Vector.<Number>
		{
			var vector : Vector.<Number> = new Vector.<Number>();
			while ( length-- > 0 )
			{
				vector.push( randomNumber() );
			}
			return vector;
		}
		
		private function equalMatrices( m1 : Matrix3D, m2 : Matrix3D ) : Boolean
		{
			var r1 : Vector.<Number> = m1.rawData;
			var r2 : Vector.<Number> = m2.rawData;
			for( var i : uint = 0; i < 16; ++i )
			{
				if( r1[i] != r2[i] ) return false;
			}
			return true;
		}
		
		private function nearEqualMatrices( m1 : Matrix3D, m2 : Matrix3D, e : Number ) : Boolean
		{
			var r1 : Vector.<Number> = m1.rawData;
			var r2 : Vector.<Number> = m2.rawData;
			for( var i : uint = 0; i < 16; ++i )
			{
				if( Math.abs( r1[i] - r2[i] ) > e )
				{
					trace( "Fails at " + i + " : " + r1[i] + " != " + r2[i] );
					return false;
				}
			}
			return true;
		}
		
		private function nearEqualVectorNums( v1 : Vector.<Number>, v2 : Vector.<Number>, e : Number ) : Boolean
		{
			if( v1.length != v2.length )
			{
				return false;
			}
			for( var i : uint = 0; i < v1.length; ++i )
			{
				if( Math.abs( v1[i] - v2[i] ) > e ) return false;
			}
			return true;
		}
		
		private function nearNumbers( a:Number, b:Number, e:Number ) : Boolean
		{
			return Math.abs( a - b ) < e;
		}
	}
