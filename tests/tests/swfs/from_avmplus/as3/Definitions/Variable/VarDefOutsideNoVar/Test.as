/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

import com.adobe.test.Assert;

// var SECTION = "Definitions";       // provide a document reference (ie, ECMA section)
// var VERSION = "Clean AS2";  // Version of JavaScript or ECMA
// var TITLE   = "Variable";       // Provide ECMA section title or a description
var BUGNUMBER = "";



    // <empty> Identifier = <empty>
    //id; // runtime error

    // <empty> Identifier = AssignmentExpression
    idAssignExpr = true;

    // <empty> VariableBindingList, Identifier = <empty>
    //id1, id2, id3; // runtime errors

    // var VariableBindingList, Identifier = AssignmentExpression
    id1AssignExpr = true, id2AssignExpr = false, id3AssignExpr = true;
    //id1AssignExprB, id2AssignExprB, id3AssignExprB = true; // runtime errors



Assert.expectEq( "Variable Definition <empty> defined inside class", 1, 1);

Assert.expectEq( "var Identifier = <empty>", "id", (id = "id", id));
Assert.expectEq( "var Identifier = AssignmentExpression", true, idAssignExpr);
Assert.expectEq( "var VariableBindingList, Identifier = <empty> [1]", true, (id1 = true, id1));
Assert.expectEq( "var VariableBindingList, Identifier = <empty> [2]", true, (id2 = true, id2));
Assert.expectEq( "var VariableBindingList, Identifier = <empty> [3]", true, (id3 = true, id3));
Assert.expectEq( "var VariableBindingList, Identifier = AssignmentExpression [1]", true, id1AssignExpr);
Assert.expectEq( "var VariableBindingList, Identifier = AssignmentExpression [2]", false, id2AssignExpr);
Assert.expectEq( "var VariableBindingList, Identifier = AssignmentExpression [3]", true, id3AssignExpr);
//Assert.expectEq( "var VariableBindingList, Identifier = AssignmentExpression [4]", undefined, id1AssignExprB);
//Assert.expectEq( "var VariableBindingList, Identifier = AssignmentExpression [5]", undefined, id2AssignExprB);
//Assert.expectEq( "var VariableBindingList, Identifier = AssignmentExpression [6]", true, id3AssignExprB);


              // displays results.
