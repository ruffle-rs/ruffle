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
var account:Account = new Account();

// delete fixed function
Assert.expectEq("delete dynamic function", false, delete account.func);


              // displays results.
