/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

// var SECTION = "Expressions";        // provide a document reference (ie, Actionscript section)
// var VERSION = "AS3";                // Version of ECMAScript or ActionScript
// var TITLE   = "delete operator";    // Provide ECMA section title or a description
var BUGNUMBER = "";


class Account {
    var name = "Jon";
    function func() { return "Account.func"; }
}

// delete non-instantiated function
Assert.expectEq("delete non-instantiated function", true, delete Account.func);
Assert.expectEq("Account.func", undefined, Account.func);


              // displays results.
