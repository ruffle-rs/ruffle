/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package VarDefPrivateStatic{

  public class VarDefPrivateStaticClass {

    // var Identifier = <empty>
    private static var id;

    // var Identifier : TypeExpression = <empty>
    private static var idTypeExpr : Boolean;

    // var Identifier = AssignmentExpression
    private static var idAssignExpr = true;

    // var Identifier : TypeExpression = AssignmentExpression
    private static var idTypeExprAssignExpr : Boolean = true;

    // var VariableBindingList, Identifier = <empty>
    private static var id1, id2, id3;

    // var VariableBindingList, Identifier : TypeExpression = <empty>
    private static var id1TypeExpr:Boolean, id2TypeExpr:Boolean, id3TypeExpr:Boolean;

    // var VariableBindingList, Identifier = AssignmentExpression
    private static var id1AssignExpr = true, id2AssignExpr = false, id3AssignExpr = true;
    private static var id1AssignExprB, id2AssignExprB, id3AssignExprB = true;

    // var VariableBindingList, Identifier : TypeExpression = AssignmentExpression
    private static var id1TypeExprAssignExpr:Boolean = true, id2TypeExprAssignExpr:Boolean = false, id3TypeExprAssignExpr:Boolean = true;
    private static var id1TypeExprAssignExprB:Boolean, id2TypeExprAssignExprB:Boolean, id3TypeExprAssignExprB:Boolean = true;

    // var Identifier, Identifier : TypeExpression
    private static var idA, idB:Boolean;

    // var Identifier, Identifier : TypeExpression = AssignmentExpression
    private static var idAAssign = false, idBAssign:Boolean = true;
    private static var idAAssignB, idBAssignB:Boolean = true;

    // var Identifier : TypeExpressionA, Identifier : TypeExpressionB = <empty>
    private static var idTypeExprA:Array, idTypeExprB:Boolean;

    // var Identifier : TypeExpressionA, Identifier : TypeExpressionB = AssignmentExpression
    private static var idTypeExprAAssign : Array = new Array(1,2,3), idTypeExprBAssign : Boolean = true;
    private static var idTypeExprAAssignB : Array, idTypeExprBAssignB : Boolean = true;

    // var Identifier, Identifier:TypeExpressionA, Identifier:TypeExpressionB = <empty>
    private static var idId, idIdTypeExprA:Array, idIdTypeExprB:Boolean;

    // var Identifier, Identifier:TypeExpressionA, Identifier:TypeExpressionB = AssignmentExpression
    private static var idIdAssign = false, idIdTypeExprAAssign:Array = new Array(1,2,3), idIdTypeExprBAssign:Boolean = true;
    private static var idIdAssignB, idIdTypeExprAAssignB:Array, idIdTypeExprBAssignB:Boolean = true;

    // *****************************************************************************************
    // access methods - only used for runtime testing
    // *****************************************************************************************

    public function getid() { return id; }
    public function setid(x) { id = x; }
    public function getidTypeExpr():Boolean { return idTypeExpr; }
    public function setidTypeExpr(x:Boolean) { idTypeExpr = x; }
    public function getidAssignExpr() { return idAssignExpr; }
    public function setidAssignExpr(x) { idAssignExpr = x; }
    public function getidTypeExprAssignExpr():Boolean { return idTypeExprAssignExpr; }
    public function setidTypeExprAssignExpr(x:Boolean) { idTypeExprAssignExpr = x; }
    public function getid1() { return id1; }
    public function setid1(x) { id1 = x; }
    public function getid2() { return id2; }
    public function setid2(x) { id2 = x; }
    public function getid3() { return id3; }
    public function setid3(x) { id3 = x; }
    public function getid1TypeExpr():Boolean { return id1TypeExpr; }
    public function setid1TypeExpr(x:Boolean) { id1TypeExpr = x; }
    public function getid2TypeExpr():Boolean { return id2TypeExpr; }
    public function setid2TypeExpr(x:Boolean) { id2TypeExpr = x; }
    public function getid3TypeExpr():Boolean { return id3TypeExpr; }
    public function setid3TypeExpr(x:Boolean) { id3TypeExpr = x; }
    public function getid1AssignExpr() { return id1AssignExpr; }
    public function getid2AssignExpr() { return id2AssignExpr; }
    public function getid3AssignExpr() { return id3AssignExpr; }
    public function getid1AssignExprB() { return id1AssignExprB; }
    public function getid2AssignExprB() { return id2AssignExprB; }
    public function getid3AssignExprB() { return id3AssignExprB; }
    public function getid1TypeExprAssignExpr():Boolean { return id1TypeExprAssignExpr; }
    public function getid2TypeExprAssignExpr():Boolean { return id2TypeExprAssignExpr; }
    public function getid3TypeExprAssignExpr():Boolean { return id3TypeExprAssignExpr; }
    public function getid1TypeExprAssignExprB():Boolean { return id1TypeExprAssignExprB; }
    public function getid2TypeExprAssignExprB():Boolean { return id2TypeExprAssignExprB; }
    public function getid3TypeExprAssignExprB():Boolean { return id3TypeExprAssignExprB; }
    public function getidA() { return idA; }
    public function setidA(x) { idA = x; }
    public function getidB():Boolean { return idB; }
    public function setidB(x:Boolean) { idB = x; }
    public function getidAAssign() { return idAAssign; }
    public function getidBAssign():Boolean { return idBAssign; }
    public function getidAAssignB() { return idAAssignB; }
    public function getidBAssignB():Boolean { return idBAssignB; }
    public function getidTypeExprA() : Array { return idTypeExprA; }
    public function setidTypeExprA(x:Array) { idTypeExprA = x; }
    public function getidTypeExprB() : Boolean { return idTypeExprB; }
    public function setidTypeExprB(x:Boolean) { idTypeExprB = x; }
    public function getidTypeExprAAssign() : Array { return idTypeExprAAssign; }
    public function getidTypeExprBAssign() : Boolean { return idTypeExprBAssign; }
    public function getidTypeExprAAssignB() : Array { return idTypeExprAAssignB; }
    public function getidTypeExprBAssignB() : Boolean { return idTypeExprBAssignB; }
    public function getidId() { return idId; }
    public function setidId(x) { idId = x; }
    public function getidIdTypeExprA() : Array { return idIdTypeExprA; }
    public function setidIdTypeExprA(x:Array) { idIdTypeExprA = x; }
    public function getidIdTypeExprB() : Boolean{ return idIdTypeExprB; }
    public function setidIdTypeExprB(x:Boolean) { idIdTypeExprB = x; }
    public function getidIdAssign() { return idIdAssign; }
    public function getidIdTypeExprAAssign() : Array { return idIdTypeExprAAssign; }
    public function getidIdTypeExprBAssign() : Boolean{ return idIdTypeExprBAssign; }
    public function getidIdAssignB() { return idIdAssignB; }
    public function getidIdTypeExprAAssignB() : Array { return idIdTypeExprAAssignB; }
    public function getidIdTypeExprBAssignB() : Boolean{ return idIdTypeExprBAssignB; }
  }
}

