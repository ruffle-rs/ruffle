/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}


import ExtError.*;
import com.adobe.test.Assert;

// var SECTION = "Definitions";       // provide a document reference (ie, ECMA section)
// var VERSION = "AS 3.0";  // Version of JavaScript or ECMA
// var TITLE   = "extend Error";       // Provide ECMA section title or a description
var BUGNUMBER = "";



var error = new CustError();
var ee = new CustEvalError();
var te = new CustTypeError();
var re = new CustReferenceError();
var ra = new CustRangeError();

Assert.expectEq( "var error = new CustError()", "Error", error.toString() );
Assert.expectEq( "var ee = new CustEvalError()", "EvalError", ee.toString() );
Assert.expectEq( "var te = new CustTypeError()", "TypeError", te.toString() );
Assert.expectEq( "var re = new CustReferenceError()", "ReferenceError", re.toString() );
Assert.expectEq( "var ra = new CustRangeError()", "RangeError", ra.toString() );

Assert.expectEq( "typeof new CustError()", "object", typeof new CustError() );
Assert.expectEq( "typeof new CustEvalError()", "object", typeof new CustEvalError() );
Assert.expectEq( "typeof new CustTypeError()", "object", typeof new CustTypeError() );
Assert.expectEq( "typeof new CustReferenceError()", "object", typeof new CustReferenceError() );
Assert.expectEq( "typeof new CustRangeError()", "object", typeof new CustRangeError() );


// dynamic Cust Error Cases

error = new CustError2();
ee = new CustEvalError2();
te = new CustTypeError2();
re = new CustReferenceError2();
ra = new CustRangeError2();

Assert.expectEq( "var error = new CustError2()", "Error", error.toString() );
Assert.expectEq( "var ee = new CustEvalError2()", "EvalError", ee.toString() );
Assert.expectEq( "var te = new CustTypeError2()", "TypeError", te.toString() );
Assert.expectEq( "var re = new CustReferenceError2()", "ReferenceError", re.toString() );
Assert.expectEq( "var ra = new CustRangeError2()", "RangeError", ra.toString() );

error = new CustError2("test");
ee = new CustEvalError2("eval error");
te = new CustTypeError2("type error");
re = new CustReferenceError2("reference error");
ra = new CustRangeError2("range error");

Assert.expectEq( "var error = new CustError2('test')", "Error: test", error.toString() );
Assert.expectEq( "var ee = new CustEvalError2('eval error')", "EvalError: eval error", ee.toString() );
Assert.expectEq( "var te = new CustTypeError2('type error')", "TypeError: type error", te.toString() );
Assert.expectEq( "var re = new CustReferenceError2('reference error')", "ReferenceError: reference error", re.toString() );
Assert.expectEq( "var ra = new CustRangeError2('range error')", "RangeError: range error", ra.toString() );

Assert.expectEq( "typeof new CustError2()", "object", typeof new CustError2() );
Assert.expectEq( "typeof new CustEvalError2()", "object", typeof new CustEvalError2() );
Assert.expectEq( "typeof new CustTypeError2()", "object", typeof new CustTypeError2() );
Assert.expectEq( "typeof new CustReferenceError2()", "object", typeof new CustReferenceError2() );
Assert.expectEq( "typeof new CustRangeError2()", "object", typeof new CustRangeError2() );

Assert.expectEq( "typeof new CustError2('test')", "object", typeof new CustError2('test') );
Assert.expectEq( "typeof new CustEvalError2('test')", "object", typeof new CustEvalError2('test') );
Assert.expectEq( "typeof new CustTypeError2('test')", "object", typeof new CustTypeError2('test') );
Assert.expectEq( "typeof new CustReferenceError2('test')", "object", typeof new CustReferenceError2('test') );
Assert.expectEq( "typeof new CustRangeError2('test')", "object", typeof new CustRangeError2('test') );


Assert.expectEq( "(err = new CustError2(), err.getClass = Object.prototype.toString, err.getClass() )",
             "[object CustError2]",
             (err = new CustError2(), err.getClass = Object.prototype.toString, err.getClass() ) );
Assert.expectEq( "(err = new CustEvalError2(), err.getClass = Object.prototype.toString, err.getClass() )",
             "[object CustEvalError2]",
             (err = new CustEvalError2(), err.getClass = Object.prototype.toString, err.getClass() ) );
Assert.expectEq( "(err = new CustTypeError2(), err.getClass = Object.prototype.toString, err.getClass() )",
             "[object CustTypeError2]",
             (err = new CustTypeError2(), err.getClass = Object.prototype.toString, err.getClass() ) );
Assert.expectEq( "(err = new CustReferenceError2(), err.getClass = Object.prototype.toString, err.getClass() )",
             "[object CustReferenceError2]",
             (err = new CustReferenceError2(), err.getClass = Object.prototype.toString, err.getClass() ) );
Assert.expectEq( "(err = new CustRangeError2(), err.getClass = Object.prototype.toString, err.getClass() )",
             "[object CustRangeError2]",
             (err = new CustRangeError2(), err.getClass = Object.prototype.toString, err.getClass() ) );

Assert.expectEq( "(err = new CustError2('test'), err.getClass = Object.prototype.toString, err.getClass() )",
             "[object CustError2]",
             (err = new CustError2('test'), err.getClass = Object.prototype.toString, err.getClass() ) );
Assert.expectEq( "(err = new CustEvalError2('test'), err.getClass = Object.prototype.toString, err.getClass() )",
             "[object CustEvalError2]",
             (err = new CustEvalError2('test'), err.getClass = Object.prototype.toString, err.getClass() ) );
Assert.expectEq( "(err = new CustTypeError2('test'), err.getClass = Object.prototype.toString, err.getClass() )",
             "[object CustTypeError2]",
             (err = new CustTypeError2('test'), err.getClass = Object.prototype.toString, err.getClass() ) );
Assert.expectEq( "(err = new CustReferenceError2('test'), err.getClass = Object.prototype.toString, err.getClass() )",
             "[object CustReferenceError2]",
             (err = new CustReferenceError2('test'), err.getClass = Object.prototype.toString, err.getClass() ) );
Assert.expectEq( "(err = new CustRangeError2('test'), err.getClass = Object.prototype.toString, err.getClass() )",
             "[object CustRangeError2]",
             (err = new CustRangeError2('test'), err.getClass = Object.prototype.toString, err.getClass() ) );



              // displays results.
