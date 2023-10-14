/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

// These test the vector optimizations introduced in bugzilla 599099.
// The tests assume that the optimizations kick in, they don't test
// that - we have benchmarks to test that elsewhere, hopefully.

// var SECTION="Vector";
// var VERSION = "AS3";



// Bugzilla 599099 will in-line reads from Vector.<int>,
// Vector.<uint>, Vector.<Number>, and Vector.<C> for some object
// class C (including "Object"), provided that the index value is
// known to be int or uint and the target of the read is compatible
// with the type that is read (for number types that means the same
// type).
//
// Bugzilla 599099 will in-line writes to Vector.<int>, Vector.<uint>,
// and Vector.<Number>, provided that the index value is known to be
// int or uint and the source of the write is the same type as the
// vector slot.

// In these tests we just want to verify that the correct values are
// read and written, that bounds checking (which is also in-lined)
// continues to be obeyed, and that the "fixed" attribute is adhered
// to.

// Vector.<int>

function verify_intvec_read(v:Vector.<int>, idx:uint, expect:int): Boolean
{
  return (v[0] == 0 &&
    v[1] == 1 &&
    v[2] == 0 &&
    v[3] == 0 &&
    v[4] == 2 &&
    v[5] == 3 &&
    v[6] == 4 &&
    v[7] == 0 &&
    v[idx] == expect);
}

function verify_intvec_write(v:Vector.<int>): Vector.<int>
{
  v[0] = 0;
  v[1] = 1;
  v[2] = 0;
  v[3] = 0;
  v[4] = 2;
  v[5] = 3;
  v[6] = 4;
  v[7] = 0;
  return v;
}

function verify_intvec_extend(v:Vector.<int>):Boolean
{
  var l:int = v.length;
  v[l] = 37;
  var k:int = v.length;
  return (k == l+1);
}

function verify_intvec_errors(v:Vector.<int>, negone:int):Boolean
{
  var l:int = v.length;
  var failed = 0;
  // read before beginning
  try { print(v[negone]); } catch (e) { failed++; }
  // read beyond end
  try { print(v[l]); } catch (e) { failed++; }
  // write before beginning
  try { v[negone] = 37; } catch (e) { failed++; }
  // write beyond end
  try { v[l] = 37; } catch (e) { failed++; }
  return (failed == 4);
}

Assert.expectEq("Vector in-lining optimization: read from Vector.<int>",
  true,
  verify_intvec_read(new <int>[0,1,0,0,2,3,4,0], 4, 2));
Assert.expectEq("Vector in-lining optimization: write to Vector.<int>",
  true,
  verify_intvec_read(verify_intvec_write(new <int>[-1, -2, -3, -4, -5, -6, -7, -8]), 5, 3));
Assert.expectEq("Vector in-lining optimization: extend extensible Vector.<int>",
  true,
  verify_intvec_extend(new <int>[0]));
Assert.expectEq("Vector in-lining optimization: error cases on Vector.<int>",
  true,
  verify_intvec_errors(new Vector.<int>(1,true), -1));

// Vector.<Number>

function verify_dblvec_read(v:Vector.<Number>, idx:uint, expect:Number): Boolean
{
  return (v[0] == 0.1 &&
    v[1] == 1.1 &&
    v[2] == 0.1 &&
    v[3] == 0.1 &&
    v[4] == 2.1 &&
    v[5] == 3.1 &&
    v[6] == 4.1 &&
    v[7] == 0.1 &&
    v[idx] == expect);
}

function verify_dblvec_write(v:Vector.<Number>): Vector.<Number>
{
  v[0] = 0.1;
  v[1] = 1.1;
  v[2] = 0.1;
  v[3] = 0.1;
  v[4] = 2.1;
  v[5] = 3.1;
  v[6] = 4.1;
  v[7] = 0.1;
  return v;
}

function verify_dblvec_extend(v:Vector.<Number>):Boolean
{
  var l:int = v.length;
  v[l] = 37.1;
  var k:int = v.length;
  return (k == l+1);
}

function verify_dblvec_errors(v:Vector.<Number>, negone:int):Boolean
{
  var l:int = v.length;
  var failed = 0;
  // read before beginning
  try { print(v[negone]); } catch (e) { failed++; }
  // read beyond end
  try { print(v[l]); } catch (e) { failed++; }
  // write before beginning
  try { v[negone] = 37.1; } catch (e) { failed++; }
  // write beyond end
  try { v[l] = 37.1; } catch (e) { failed++; }
  return (failed == 4);
}

Assert.expectEq("Vector in-lining optimization: read from Vector.<Number>",
  true,
  verify_dblvec_read(new <Number>[0.1, 1.1, 0.1, 0.1, 2.1, 3.1, 4.1, 0.1], 4, 2.1));
Assert.expectEq("Vector in-lining optimization: write to Vector.<Number>",
  true,
  verify_dblvec_read(verify_dblvec_write(new <Number>[-1, -2, -3, -4, -5, -6, -7, -8]), 5, 3.1));
Assert.expectEq("Vector in-lining optimization: extend extensible Vector.<Number>",
  true,
  verify_dblvec_extend(new <Number>[0]));
Assert.expectEq("Vector in-lining optimization: error cases on Vector.<Number>",
  true,
  verify_dblvec_errors(new Vector.<Number>(1,true), -1));

// Vector.<uint>

function verify_uintvec_read(v:Vector.<uint>, idx:uint, expect:uint): Boolean
{
  return (v[0] == 0 &&
    v[1] == 1 &&
    v[2] == 0 &&
    v[3] == 0 &&
    v[4] == 2 &&
    v[5] == 3 &&
    v[6] == 4 &&
    v[7] == 0 &&
    v[idx] == expect);
}

function verify_uintvec_write(v:Vector.<uint>): Vector.<uint>
{
  v[0] = 0;
  v[1] = 1;
  v[2] = 0;
  v[3] = 0;
  v[4] = 2;
  v[5] = 3;
  v[6] = 4;
  v[7] = 0;
  return v;
}

function verify_uintvec_extend(v:Vector.<uint>):Boolean
{
  var l:int = v.length;
  v[l] = 37;
  var k:int = v.length;
  return (k == l+1);
}

function verify_uintvec_errors(v:Vector.<uint>, negone:int):Boolean
{
  var l:int = v.length;
  var failed = 0;
  // read before beginning
  try { print(v[negone]); } catch (e) { failed++; }
  // read beyond end
  try { print(v[l]); } catch (e) { failed++; }
  // write before beginning
  try { v[negone] = 37; } catch (e) { failed++; }
  // write beyond end
  try { v[l] = 37; } catch (e) { failed++; }
  return (failed == 4);
}

Assert.expectEq("Vector in-lining optimization: read from Vector.<uint>",
  true,
  verify_uintvec_read(new <uint>[0,1,0,0,2,3,4,0], 4, 2));
Assert.expectEq("Vector in-lining optimization: write to Vector.<uint>",
  true,
  verify_uintvec_read(verify_uintvec_write(new <uint>[-1, -2, -3, -4, -5, -6, -7, -8]), 5, 3)); // well, conversion works...
Assert.expectEq("Vector in-lining optimization: extend extensible Vector.<uint>",
  true,
  verify_uintvec_extend(new <uint>[0]));
Assert.expectEq("Vector in-lining optimization: error cases on Vector.<int>",
  true,
  verify_uintvec_errors(new Vector.<uint>(1,true), -1));

// Vector.<Object>
//
// Currently only "read" tests because the optimization in bugzilla 599099 does not optimize
// the "write" case here.

var o0: Object = {};
var o1: Object = {};
var o2: Object = {};
var o3: Object = {};

function verify_objvec_read(v:Vector.<Object>, idx:uint, expect:Object): Boolean
{
  return (v[0] == o0 &&
    v[1] == o1 &&
    v[2] == o2 &&
    v[3] == o3 &&
    v[idx] == expect);
}

function verify_objvec_errors(v:Vector.<Object>, negone:int):Boolean
{
  var l:int = v.length;
  var failed = 0;
  // read before beginning
  try { print(v[negone]); } catch (e) { failed++; }
  // read beyond end
  try { print(v[l]); } catch (e) { failed++; }
  return (failed == 2);
}

Assert.expectEq("Vector in-lining optimization: read from Vector.<Object>",
  true,
  verify_objvec_read(new <Object>[o0, o1, o2, o3], 2, o2));
Assert.expectEq("Vector in-lining optimization: error cases on Vector.<Object>",
  true,
  verify_objvec_errors(new <Object>[o0], -1));
