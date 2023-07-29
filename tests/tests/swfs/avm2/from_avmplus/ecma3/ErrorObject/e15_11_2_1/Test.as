/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

// var SECTION = "ErrorObjects";       // provide a document reference (ie, Actionscript section)
// var VERSION = "ES3";        // Version of ECMAScript or ActionScript
// var TITLE   = "new Error(message)";       // Provide ECMA section title or a description
var BUGNUMBER = "";


var testcases = getTestCases();


              // displays results.
              
function getTestCases() {
    var array = new Array();
    var item = 0;
    var error = new Error();
    var ee = new EvalError();
    var te = new TypeError();
    var re = new ReferenceError();
    var ra = new RangeError();

    array[item++] = Assert.expectEq( "var error = new Error()", "Error", error.toString() );
    array[item++] = Assert.expectEq( "var ee = new EvalError()", "EvalError", ee.toString() );
    array[item++] = Assert.expectEq( "var te = new TypeError()", "TypeError", te.toString() );
    array[item++] = Assert.expectEq( "var re = new ReferenceError()", "ReferenceError", re.toString() );
    array[item++] = Assert.expectEq( "var ra = new RangeError()", "RangeError", ra.toString() );

    error = new Error("test");
    ee = new EvalError("eval error");
    te = new TypeError("type error");
    re = new ReferenceError("reference error");
    ra = new RangeError("range error");

    array[item++] = Assert.expectEq( "var error = new Error('test')", "Error: test", error.toString() );
    array[item++] = Assert.expectEq( "var ee = new EvalError('eval error')", "EvalError: eval error", ee.toString() );
    array[item++] = Assert.expectEq( "var te = new TypeError('type error')", "TypeError: type error", te.toString() );
    array[item++] = Assert.expectEq( "var re = new ReferenceError('reference error')", "ReferenceError: reference error", re.toString() );
    array[item++] = Assert.expectEq( "var ra = new RangeError('range error')", "RangeError: range error", ra.toString() );

    array[item++] = Assert.expectEq( "typeof new Error()", "object", typeof new Error() );
    array[item++] = Assert.expectEq( "typeof new EvalError()", "object", typeof new EvalError() );
    array[item++] = Assert.expectEq( "typeof new TypeError()", "object", typeof new TypeError() );
    array[item++] = Assert.expectEq( "typeof new ReferenceError()", "object", typeof new ReferenceError() );
    array[item++] = Assert.expectEq( "typeof new RangeError()", "object", typeof new RangeError() );

    array[item++] = Assert.expectEq( "typeof new Error('test')", "object", typeof new Error('test') );
    array[item++] = Assert.expectEq( "typeof new EvalError('test')", "object", typeof new EvalError('test') );
    array[item++] = Assert.expectEq( "typeof new TypeError('test')", "object", typeof new TypeError('test') );
    array[item++] = Assert.expectEq( "typeof new ReferenceError('test')", "object", typeof new ReferenceError('test') );
    array[item++] = Assert.expectEq( "typeof new RangeError('test')", "object", typeof new RangeError('test') );

    array[item++] = Assert.expectEq( "(err = new Error(), err.getClass = Object.prototype.toString, err.getClass() )",
                 "[object Error]",
                 (err = new Error(), err.getClass = Object.prototype.toString, err.getClass() ) );
    array[item++] = Assert.expectEq( "(err = new EvalError(), err.getClass = Object.prototype.toString, err.getClass() )",
                 "[object EvalError]",
                 (err = new EvalError(), err.getClass = Object.prototype.toString, err.getClass() ) );
    array[item++] = Assert.expectEq( "(err = new TypeError(), err.getClass = Object.prototype.toString, err.getClass() )",
                 "[object TypeError]",
                 (err = new TypeError(), err.getClass = Object.prototype.toString, err.getClass() ) );
    array[item++] = Assert.expectEq( "(err = new ReferenceError(), err.getClass = Object.prototype.toString, err.getClass() )",
                 "[object ReferenceError]",
                 (err = new ReferenceError(), err.getClass = Object.prototype.toString, err.getClass() ) );
    array[item++] = Assert.expectEq( "(err = new RangeError(), err.getClass = Object.prototype.toString, err.getClass() )",
                 "[object RangeError]",
                 (err = new RangeError(), err.getClass = Object.prototype.toString, err.getClass() ) );

    array[item++] = Assert.expectEq( "(err = new Error('test'), err.getClass = Object.prototype.toString, err.getClass() )",
                 "[object Error]",
                 (err = new Error('test'), err.getClass = Object.prototype.toString, err.getClass() ) );
    array[item++] = Assert.expectEq( "(err = new EvalError('test'), err.getClass = Object.prototype.toString, err.getClass() )",
                 "[object EvalError]",
                 (err = new EvalError('test'), err.getClass = Object.prototype.toString, err.getClass() ) );
    array[item++] = Assert.expectEq( "(err = new TypeError('test'), err.getClass = Object.prototype.toString, err.getClass() )",
                 "[object TypeError]",
                 (err = new TypeError('test'), err.getClass = Object.prototype.toString, err.getClass() ) );
    array[item++] = Assert.expectEq( "(err = new ReferenceError('test'), err.getClass = Object.prototype.toString, err.getClass() )",
                 "[object ReferenceError]",
                 (err = new ReferenceError('test'), err.getClass = Object.prototype.toString, err.getClass() ) );
    array[item++] = Assert.expectEq( "(err = new RangeError('test'), err.getClass = Object.prototype.toString, err.getClass() )",
                 "[object RangeError]",
                 (err = new RangeError('test'), err.getClass = Object.prototype.toString, err.getClass() ) );

    return array;
}
