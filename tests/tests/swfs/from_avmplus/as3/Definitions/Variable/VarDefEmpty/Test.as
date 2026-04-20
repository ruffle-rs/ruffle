/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

// var SECTION = "Definitions";       // provide a document reference (ie, ECMA section)
// var VERSION = "Clean AS2";  // Version of JavaScript or ECMA
// var TITLE   = "Variable Definition";       // Provide ECMA section title or a description
var BUGNUMBER = "";



import VarDefEmptyPkg.*;

import com.adobe.test.Assert;
var VARDEFEMPTY = new VarDefEmpty();
Assert.expectEq( "Variable Definition <empty> defined inside class", 1, 1);

Assert.expectEq( "var Identifier = <empty>", "id", (VARDEFEMPTY.setid("id"), VARDEFEMPTY.getid()));
Assert.expectEq( "var Identifier : TypeExpression = <empty>", true, (VARDEFEMPTY.setidTypeExpr(true),
                                                                 VARDEFEMPTY.getidTypeExpr()));
Assert.expectEq( "var Identifier = AssignmentExpression", true, VARDEFEMPTY.getidAssignExpr());
Assert.expectEq( "var Identifier : TypeExpression = AssignmentExpression", true, VARDEFEMPTY.getidTypeExprAssignExpr());
Assert.expectEq( "var VariableBindingList, Identifier = <empty> [1]", true, (VARDEFEMPTY.setid1(true),
                                                                         VARDEFEMPTY.getid1()));
Assert.expectEq( "var VariableBindingList, Identifier = <empty> [2]", true, (VARDEFEMPTY.setid2(true),
                                                                         VARDEFEMPTY.getid2()));
Assert.expectEq( "var VariableBindingList, Identifier = <empty> [3]", true, (VARDEFEMPTY.setid3(true),
                                                                         VARDEFEMPTY.getid3()));
Assert.expectEq( "var VariableBindingList, Identifier : TypeExpression = <empty> [1]", true, (VARDEFEMPTY.setid1TypeExpr(true),
                                                                                          VARDEFEMPTY.getid1TypeExpr()));
Assert.expectEq( "var VariableBindingList, Identifier : TypeExpression = <empty> [2]", true, (VARDEFEMPTY.setid2TypeExpr(true),
                                                                                          VARDEFEMPTY.getid2TypeExpr()));
Assert.expectEq( "var VariableBindingList, Identifier : TypeExpression = <empty> [3]", true, (VARDEFEMPTY.setid3TypeExpr(true),
                                                                                          VARDEFEMPTY.getid3TypeExpr()));
Assert.expectEq( "var VariableBindingList, Identifier = AssignmentExpression [1]", true, VARDEFEMPTY.getid1AssignExpr());
Assert.expectEq( "var VariableBindingList, Identifier = AssignmentExpression [2]", false, VARDEFEMPTY.getid2AssignExpr());
Assert.expectEq( "var VariableBindingList, Identifier = AssignmentExpression [3]", true, VARDEFEMPTY.getid3AssignExpr());
Assert.expectEq( "var VariableBindingList, Identifier = AssignmentExpression [4]", undefined, VARDEFEMPTY.getid1AssignExprB());
Assert.expectEq( "var VariableBindingList, Identifier = AssignmentExpression [5]", undefined, VARDEFEMPTY.getid2AssignExprB());
Assert.expectEq( "var VariableBindingList, Identifier = AssignmentExpression [6]", true, VARDEFEMPTY.getid3AssignExprB());
Assert.expectEq( "var VariableBindingList, Identifier : TypeExpression = AssignmentExpression [1]", true, VARDEFEMPTY.getid1TypeExprAssignExpr());
Assert.expectEq( "var VariableBindingList, Identifier : TypeExpression = AssignmentExpression [2]", false, VARDEFEMPTY.getid2TypeExprAssignExpr());
Assert.expectEq( "var VariableBindingList, Identifier : TypeExpression = AssignmentExpression [3]", true, VARDEFEMPTY.getid3TypeExprAssignExpr());
Assert.expectEq( "var VariableBindingList, Identifier : TypeExpression = AssignmentExpression [4]", false, VARDEFEMPTY.getid1TypeExprAssignExprB());
Assert.expectEq( "var VariableBindingList, Identifier : TypeExpression = AssignmentExpression [5]", false, VARDEFEMPTY.getid2TypeExprAssignExprB());
Assert.expectEq( "var VariableBindingList, Identifier : TypeExpression = AssignmentExpression [6]", true, VARDEFEMPTY.getid3TypeExprAssignExprB());
Assert.expectEq( "var Identifier, Identifier : TypeExpression = <empty> [1]", true, (VARDEFEMPTY.setidA(true),
                                                                                 VARDEFEMPTY.getidA()));
Assert.expectEq( "var Identifier, Identifier : TypeExpression = <empty> [2]", true, (VARDEFEMPTY.setidB(true),
                                                                                 VARDEFEMPTY.getidB()));
Assert.expectEq( "var Identifier, Identifier : TypeExpression = AssignmentExpression [1]", false, VARDEFEMPTY.getidAAssign() )
Assert.expectEq( "var Identifier, Identifier : TypeExpression = AssignmentExpression [2]", true, VARDEFEMPTY.getidBAssign() )
Assert.expectEq( "var Identifier, Identifier : TypeExpression = AssignmentExpression [3]", undefined, VARDEFEMPTY.getidAAssignB() )
Assert.expectEq( "var Identifier, Identifier : TypeExpression = AssignmentExpression [4]", true, VARDEFEMPTY.getidBAssignB() )
var arr = new Array(1,2,3);
Assert.expectEq( "var Identifier : TypeExpressionA, Identifier : TypeExpressionB = <empty> [1]", arr, (VARDEFEMPTY.setidTypeExprA(arr),
                                                                                                   VARDEFEMPTY.getidTypeExprA()) );
Assert.expectEq( "var Identifier : TypeExpressionA, Identifier : TypeExpressionB = <empty> [2]", false, (VARDEFEMPTY.setidTypeExprB(false),
                                                                                                     VARDEFEMPTY.getidTypeExprB()) );
Assert.expectEq( "var Identifier : TypeExpressionA, Identifier : TypeExpressionB = AssignmentExpression [1]", arr.toString(), VARDEFEMPTY.getidTypeExprAAssign().toString())
Assert.expectEq( "var Identifier : TypeExpressionA, Identifier : TypeExpressionB = AssignmentExpression [2]", true, VARDEFEMPTY.getidTypeExprBAssign() )
Assert.expectEq( "var Identifier : TypeExpressionA, Identifier : TypeExpressionB = AssignmentExpression [3]", null, VARDEFEMPTY.getidTypeExprAAssignB())
Assert.expectEq( "var Identifier : TypeExpressionA, Identifier : TypeExpressionB = AssignmentExpression [4]", true, VARDEFEMPTY.getidTypeExprBAssignB() )
Assert.expectEq( "var Identifier, Identifier : TypeExpressionA, Identifier : TypeExpressionB = <empty> [1]", true, (VARDEFEMPTY.setidId(true),
                                                                                                                VARDEFEMPTY.getidId()) );
Assert.expectEq( "var Identifier, Identifier : TypeExpressionA, Identifier : TypeExpressionB = <empty> [2]", arr, (VARDEFEMPTY.setidIdTypeExprA(arr),
                                                                                                               VARDEFEMPTY.getidIdTypeExprA()) );
Assert.expectEq( "var Identifier, Identifier : TypeExpressionA, Identifier : TypeExpressionB = <empty> [3]", false, (VARDEFEMPTY.setidIdTypeExprB(false),
                                                                                                               VARDEFEMPTY.getidIdTypeExprB()) );
Assert.expectEq( "var Identififer, Identifier : TypeExpressionA, Identifier : TypeExpressionB = AssignmentExpression [1]", false, VARDEFEMPTY.getidIdAssign());

Assert.expectEq( "var Identififer, Identifier : TypeExpressionA, Identifier : TypeExpressionB = AssignmentExpression [2]", arr.toString(), VARDEFEMPTY.getidIdTypeExprAAssign()+"");
Assert.expectEq( "var Identififer, Identifier : TypeExpressionA, Identifier : TypeExpressionB = AssignmentExpression [3]", true, VARDEFEMPTY.getidIdTypeExprBAssign());
Assert.expectEq( "var Identififer, Identifier : TypeExpressionA, Identifier : TypeExpressionB = AssignmentExpression [4]", undefined, VARDEFEMPTY.getidIdAssignB());
Assert.expectEq( "var Identififer, Identifier : TypeExpressionA, Identifier : TypeExpressionB = AssignmentExpression [5]", null, VARDEFEMPTY.getidIdTypeExprAAssignB());
Assert.expectEq( "var Identififer, Identifier : TypeExpressionA, Identifier : TypeExpressionB = AssignmentExpression [6]", true, VARDEFEMPTY.getidIdTypeExprBAssignB());



              // displays results.
