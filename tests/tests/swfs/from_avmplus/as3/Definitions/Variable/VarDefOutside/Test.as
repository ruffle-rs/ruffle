/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

import com.adobe.test.Assert;

// var SECTION = "Definitions";       // provide a document reference (ie, ECMA section)
// var VERSION = "Clean AS2";  // Version of JavaScript or ECMA
// var TITLE   = "Variable";       // Provide ECMA section title or a description
var BUGNUMBER = "";



    // var Identifier = <empty>
    var id;

    // var Identifier : TypeExpression = <empty>
    var idTypeExpr : Boolean;

    // var Identifier = AssignmentExpression
    var idAssignExpr = true;

    // var Identifier : TypeExpression = AssignmentExpression
    var idTypeExprAssignExpr : Boolean = true;

    // var VariableBindingList, Identifier = <empty>
    var id1, id2, id3;

    // var VariableBindingList, Identifier : TypeExpression = <empty>
    var id1TypeExpr:Boolean, id2TypeExpr:Boolean, id3TypeExpr:Boolean;

    // var VariableBindingList, Identifier = AssignmentExpression
    // Bug 117477
    //var id1AssignExpr = true, id2AssignExpr = false, id3AssignExpr = true;
    var id1AssignExprB, id2AssignExprB, id3AssignExprB = true;

    // var VariableBindingList, Identifier : TypeExpression = AssignmentExpression
    // Bug 117477
    //var id1TypeExprAssignExpr:Boolean = true, id2TypeExprAssignExpr:Boolean = false, id3TypeExprAssignExpr:Boolean = true;
    var id1TypeExprAssignExprB:Boolean, id2TypeExprAssignExprB:Boolean, id3TypeExprAssignExprB:Boolean = true;

    // var Identifier, Identifier : TypeExpression
    var idA, idB:Boolean;

    // var Identifier, Identifier : TypeExpression = AssignmentExpression
    // Bug 117477
    //var idAAssign = false, idBAssign:Boolean = true;
    //var idAAssignB, idBAssignB:Boolean = true;

    // var Identifier : TypeExpressionA, Identifier : TypeExpressionB = <empty>
    var idTypeExprA:Array, idTypeExprB:Boolean;

    // var Identifier : TypeExpressionA, Identifier : TypeExpressionB = AssignmentExpression
    // Bug 117477
    //var idTypeExprAAssign : Array = new Array(1,2,3), idTypeExprBAssign : Boolean = true;
    //var idTypeExprAAssignB : Array, idTypeExprBAssignB : Boolean = true;

    // var Identifier, Identifier:TypeExpressionA, Identifier:TypeExpressionB = <empty>
    var idId, idIdTypeExprA:Array, idIdTypeExprB:Boolean;

    // var Identifier, Identifier:TypeExpressionA, Identifier:TypeExpressionB = AssignmentExpression
    // Bug 117477
    //var idIdAssign = false, idIdTypeExprAAssign:Array = new Array(1,2,3), idIdTypeExprBAssign:Boolean = true;
    //var idIdAssignB, idIdTypeExprAAssignB:Array, idIdTypeExprBAssignB:Boolean = true;

Assert.expectEq( "Variable Definition <empty> defined inside class", 1, 1);

Assert.expectEq( "var Identifier = <empty>", "id", (id = "id", id));
Assert.expectEq( "var Identifier : TypeExpression = <empty>", true, (idTypeExpr = true, idTypeExpr ));
Assert.expectEq( "var Identifier = AssignmentExpression", true, idAssignExpr);
Assert.expectEq( "var Identifier : TypeExpression = AssignmentExpression", true, idTypeExprAssignExpr);
Assert.expectEq( "var VariableBindingList, Identifier = <empty> [1]", true, (id1 = true, id1));
Assert.expectEq( "var VariableBindingList, Identifier = <empty> [2]", true, (id2 = true, id2));
Assert.expectEq( "var VariableBindingList, Identifier = <empty> [3]", true, (id3 = true, id3));
//Assert.expectEq( "var VariableBindingList, Identifier : TypeExpression = <empty> [1]", true, (id1TypeExpr = true,
//                                                                                          id1TypeExpr));
Assert.expectEq( "var VariableBindingList, Identifier : TypeExpression = <empty> [2]", true, (id2TypeExpr = true,
                                                                                          id2TypeExpr));
Assert.expectEq( "var VariableBindingList, Identifier : TypeExpression = <empty> [3]", true, (id3TypeExpr = true,
                                                                                          id3TypeExpr));
//Assert.expectEq( "var VariableBindingList, Identifier = AssignmentExpression [1]", true, id1AssignExpr);
//Assert.expectEq( "var VariableBindingList, Identifier = AssignmentExpression [2]", false, id2AssignExpr);
//Assert.expectEq( "var VariableBindingList, Identifier = AssignmentExpression [3]", true, id3AssignExpr);
Assert.expectEq( "var VariableBindingList, Identifier = AssignmentExpression [4]", undefined, id1AssignExprB);
Assert.expectEq( "var VariableBindingList, Identifier = AssignmentExpression [5]", undefined, id2AssignExprB);
Assert.expectEq( "var VariableBindingList, Identifier = AssignmentExpression [6]", true, id3AssignExprB);
//Assert.expectEq( "var VariableBindingList, Identifier : TypeExpression = AssignmentExpression [1]", true, id1TypeExprAssignExpr);
//Assert.expectEq( "var VariableBindingList, Identifier : TypeExpression = AssignmentExpression [2]", false, id2TypeExprAssignExpr);
//Assert.expectEq( "var VariableBindingList, Identifier : TypeExpression = AssignmentExpression [3]", true, id3TypeExprAssignExpr);
Assert.expectEq( "var VariableBindingList, Identifier : TypeExpression = AssignmentExpression [4]", false, id1TypeExprAssignExprB);
Assert.expectEq( "var VariableBindingList, Identifier : TypeExpression = AssignmentExpression [5]", false, id2TypeExprAssignExprB);
Assert.expectEq( "var VariableBindingList, Identifier : TypeExpression = AssignmentExpression [6]", true, id3TypeExprAssignExprB);
Assert.expectEq( "var Identifier, Identifier : TypeExpression = <empty> [1]", true, (idA = true,
                                                                                 idA));
Assert.expectEq( "var Identifier, Identifier : TypeExpression = <empty> [2]", true, (idB = true,
                                                                                 idB));
//Assert.expectEq( "var Identifier, Identifier : TypeExpression = AssignmentExpression [1]", false, idAAssign );
//Assert.expectEq( "var Identifier, Identifier : TypeExpression = AssignmentExpression [2]", true, idBAssign );
//Assert.expectEq( "var Identifier, Identifier : TypeExpression = AssignmentExpression [3]", undefined, idAAssignB );
//Assert.expectEq( "var Identifier, Identifier : TypeExpression = AssignmentExpression [4]", true, idBAssignB );
var arr = new Array(1,2,3);
Assert.expectEq( "var Identifier : TypeExpressionA, Identifier : TypeExpressionB = <empty> [1]", arr, (idTypeExprA = arr,
                                                                                                   idTypeExprA ));
Assert.expectEq( "var Identifier : TypeExpressionA, Identifier : TypeExpressionB = <empty> [2]", false, (idTypeExprB = false,
                                                                                                     idTypeExprB));
//Assert.expectEq( "var Identifier : TypeExpressionA, Identifier : TypeExpressionB = AssignmentExpression [1]", arr.toString(), idTypeExprAAssign.toString())
//Assert.expectEq( "var Identifier : TypeExpressionA, Identifier : TypeExpressionB = AssignmentExpression [2]", true, idTypeExprBAssign )
//Assert.expectEq( "var Identifier : TypeExpressionA, Identifier : TypeExpressionB = AssignmentExpression [3]", undefined, idTypeExprAAssignB)
//Assert.expectEq( "var Identifier : TypeExpressionA, Identifier : TypeExpressionB = AssignmentExpression [4]", true, idTypeExprBAssignB )
/*Assert.expectEq( "var Identifier, Identifier : TypeExpressionA, Identifier : TypeExpressionB = <empty> [1]", true, (idId = true,
                                                                                                                idId) );
Assert.expectEq( "var Identifier, Identifier : TypeExpressionA, Identifier : TypeExpressionB = <empty> [2]", arr, (idIdTypeExprA = arr,
                                                                                                               idIdTypeExprA) );
Assert.expectEq( "var Identifier, Identifier : TypeExpressionA, Identifier : TypeExpressionB = <empty> [3]", false, (idIdTypeExprB = false,
                                                                                                                 idIdTypeExprB) );
Assert.expectEq( "var Identififer, Identifier : TypeExpressionA, Identifier : TypeExpressionB = AssignmentExpression [1]", false, idIdAssign);
Assert.expectEq( "var Identififer, Identifier : TypeExpressionA, Identifier : TypeExpressionB = AssignmentExpression [2]", arr.toString(), idIdTypeExprAAssign.toString());
Assert.expectEq( "var Identififer, Identifier : TypeExpressionA, Identifier : TypeExpressionB = AssignmentExpression [3]", true, idIdTypeExprBAssign);
Assert.expectEq( "var Identififer, Identifier : TypeExpressionA, Identifier : TypeExpressionB = AssignmentExpression [4]", undefined, idIdAssignB);
Assert.expectEq( "var Identififer, Identifier : TypeExpressionA, Identifier : TypeExpressionB = AssignmentExpression [5]", undefined, idIdTypeExprAAssignB);
Assert.expectEq( "var Identififer, Identifier : TypeExpressionA, Identifier : TypeExpressionB = AssignmentExpression [6]", true, idIdTypeExprBAssignB);
*/

              // displays results.
