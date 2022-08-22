package  {
	public class Test {

	}	
}

import flash.geom.Matrix;
import flash.geom.Point;
import flash.geom.Vector3D;

trace("new matrix", new Matrix());
trace("new Matrix(1)", new Matrix(1));
trace("new Matrix(1, 2)", new Matrix(1, 2));
trace("new Matrix(1, 2, 3)", new Matrix(1, 2, 3));
trace("new Matrix(1, 2, 3, 4)", new Matrix(1, 2, 3, 4));
trace("new Matrix(1, 2, 3, 4, 5)", new Matrix(1, 2, 3, 4, 5));
trace("new Matrix(1, 2, 3, 4, 5, 6)", new Matrix(1, 2, 3, 4, 5, 6));

// cannot use NaN values with approximate tester
// var temp = {}; // to please the compiler
// trace("// new Matrix(1, 2, 3, {})");
// trace(new Matrix(1, 2, 3, temp));
trace("");

trace("// new Matrix(1, 2, 3, 4, 5, 6) .identity()");
var matrix = new Matrix(1, 2, 3, 4, 5, 6);
matrix.identity();
trace(matrix);
trace("");
trace("");

trace("/// Clones");
matrix = new Matrix(1, 2, 3, 4, 5, 6);
var cloned = matrix.clone();
trace("// matrix");
trace(matrix);
trace("");

trace("// cloned");
trace(cloned);
trace("");

trace("// matrix === cloned");
trace(matrix === cloned);
trace("");
trace("");

// cannot use NaN values with approximate tester
// var temp = {};
// matrix = new Matrix(1, 2, temp, 4, 5, 6);
// cloned = matrix.clone();
// trace("// matrix");
// trace(matrix);
// trace("");

trace("// cloned");
trace(cloned);
trace("");

trace("// matrix === cloned");
trace(matrix === cloned);
trace("");
trace("");


trace("/// scale");
trace("// matrix");
matrix = new Matrix();
trace(matrix);
trace("");

trace("// matrix.scale(3, 5)");
matrix.scale(3, 5);
trace(matrix);
trace("");
trace("");

trace("// matrix");
matrix = new Matrix(2, 0, 0, 2, 100, 100);
trace(matrix);
trace("");

trace("// matrix.scale(7, 11)");
matrix.scale(7, 11);
trace(matrix);
trace("");
trace("");

trace("// matrix");
matrix = new Matrix(1, 2, 3, 4, 5, 6);
trace(matrix);
trace("");

trace("// matrix.scale(13, 17)");
matrix.scale(13, 17);
trace(matrix);
trace("");
trace("");

trace("/// rotate");
matrix = new Matrix();
trace(matrix);
trace("");

trace("// matrix.rotate(0)");
matrix.rotate(0);
trace(matrix);
trace("");
trace("");

trace("/// rotate");
trace("// matrix");
matrix = new Matrix();
trace(matrix);
trace("");

trace("// matrix.rotate(0.5)");
matrix.rotate(0.5);
trace(matrix);
trace("");
trace("");

trace("// matrix");
matrix = new Matrix(1, 2, 3, 4, 5, 6);
trace(matrix);
trace("");

trace("// matrix.rotate(0)");
matrix.rotate(0);
trace(matrix);
trace("");
trace("");

trace("// matrix");
matrix = new Matrix(1, 2, 3, 4, 5, 6);
trace(matrix);
trace("");

trace("// matrix.rotate((90/180)*Math.PI)");
matrix.rotate((90/180)*Math.PI);
trace(matrix);
trace("");
trace("");

trace("/// translate");
trace("// matrix");
matrix = new Matrix();

trace("// matrix.translate(3, 5)");
matrix.translate(3, 5);
trace(matrix);
trace("");
trace("");

trace("// matrix");
matrix = new Matrix(2, 0, 0, 2, 100, 100);

trace("// matrix.translate(7, 11)");
matrix.translate(7, 11);
trace(matrix);
trace("");
trace("");

trace("/// concat");
var scale = new Matrix();
scale.scale(3, 5);
var translate = new Matrix();
translate.translate(7, 9);
var rotate = new Matrix();
rotate.rotate(Math.PI / 2);
matrix = new Matrix(11, 13, 17, 19, 23, 29);

trace("matrix:", matrix);
trace("scale(3, 5):", scale);
trace("translate(7, 9):", translate);
trace("rotate(Math.PI / 2):", rotate);
trace("");

var result = matrix.clone();

trace("//double transform");
result = scale.clone();
result.concat(translate);
trace("//scale + translate", result);

result = translate.clone();
result.concat(scale);
trace("//translate + scale", result);

result = scale.clone();
result.concat(rotate);
trace("//scale + rotate", result);

result = rotate.clone();
result.concat(scale);
trace("//rotate + scale", result);

result = translate.clone();
result.concat(rotate);
trace("//translate + rotate", result);

result = rotate.clone();
result.concat(translate);
trace("//rotate + translate", result);
trace("");

trace("//triple transform");
result = scale.clone();
result.concat(translate);
result.concat(rotate);
trace("//scale + translate + rotate", result);

result = scale.clone();
result.concat(rotate);
result.concat(translate);
trace("//scale + rotate + translate", result);

result = translate.clone();
result.concat(scale);
result.concat(rotate);
trace("//translate + scale + rotate", result);

result = translate.clone();
result.concat(rotate);
result.concat(scale);
trace("//translate + rotate + scale", result);

result = rotate.clone();
result.concat(translate);
result.concat(scale);
trace("//rotate + translate + scale", result);

result = rotate.clone();
result.concat(scale);
result.concat(translate);
trace("//rotate + scale + translate", result);
trace("");

trace("//right application");
trace("");

trace("//single transform");

result = matrix.clone();
result.concat(scale);
trace("//matrix + scale", result);

result = matrix.clone();
result.concat(translate);
trace("//matrix + translate", result);

result = matrix.clone();
result.concat(rotate);
trace("//matrix + rotate", result);
trace("");

trace("//double transform");

result = matrix.clone();
result.concat(scale);
result.concat(translate);
trace("//matrix + scale + translate", result);

result = matrix.clone();
result.concat(translate);
result.concat(scale);
trace("//matrix + translate + scale", result);

result = matrix.clone();
result.concat(scale);
result.concat(rotate);
trace("//matrix + scale + rotate", result);

result = matrix.clone();
result.concat(rotate);
result.concat(scale);
trace("//matrix + rotate + scale", result);

result = matrix.clone();
result.concat(translate);
result.concat(rotate);
trace("//matrix + translate + rotate", result);

result = matrix.clone();
result.concat(rotate);
result.concat(translate);
trace("//matrix + rotate + translate", result);
trace("");

trace("//triple transform");

result = matrix.clone();
result.concat(scale);
result.concat(translate);
result.concat(rotate);
trace("//matrix + scale + translate + rotate", result);

result = matrix.clone();
result.concat(scale);
result.concat(rotate);
result.concat(translate);
trace("//matrix + scale + rotate + translate", result);

result = matrix.clone();
result.concat(translate);
result.concat(scale);
result.concat(rotate);
trace("//matrix + translate + scale + rotate", result);

result = matrix.clone();
result.concat(translate);
result.concat(rotate);
result.concat(scale);
trace("//matrix + translate + rotate + scale", result);

result = matrix.clone();
result.concat(rotate);
result.concat(translate);
result.concat(scale);
trace("//matrix + rotate + translate + scale", result);

result = matrix.clone();
result.concat(rotate);
result.concat(scale);
result.concat(translate);
trace("//matrix + rotate + scale + translate", result);
trace("");

trace("//left application");
trace("");

trace("//single transform");

result = scale.clone();
result.concat(matrix);
trace("//scale + matrix", result);

result = translate.clone();
result.concat(matrix);
trace("//translate + matrix", result);

result = rotate.clone();
result.concat(matrix);
trace("//rotate + matrix", result);
trace("");

trace("//double transform");

result = scale.clone();
result.concat(translate);
result.concat(matrix);
trace("//scale + translate + matrix", result);

result = translate.clone();
result.concat(scale);
result.concat(matrix);
trace("//translate + scale + matrix", result);

result = scale.clone();
result.concat(rotate);
result.concat(matrix);
trace("//scale + rotate + matrix", result);

result = rotate.clone();
result.concat(scale);
result.concat(matrix);
trace("//rotate + scale + matrix", result);

result = translate.clone();
result.concat(rotate);
result.concat(matrix);
trace("//translate + rotate + matrix", result);

result = rotate.clone();
result.concat(translate);
result.concat(matrix);
trace("//rotate + translate + matrix", result);
trace("");

trace("//triple transform");

result = scale.clone();
result.concat(translate);
result.concat(rotate);
result.concat(matrix);
trace("//scale + translate + rotate + matrix", result);

result = scale.clone();
result.concat(rotate);
result.concat(translate);
result.concat(matrix);
trace("//scale + rotate + translate + matrix", result);

result = translate.clone();
result.concat(scale);
result.concat(rotate);
result.concat(matrix);
trace("//translate + scale + rotate + matrix", result);

result = translate.clone();
result.concat(rotate);
result.concat(scale);
result.concat(matrix);
trace("//translate + rotate + scale + matrix", result);

result = rotate.clone();
result.concat(translate);
result.concat(scale);
result.concat(matrix);
trace("//rotate + translate + scale + matrix", result);

result = rotate.clone();
result.concat(scale);
result.concat(translate);
result.concat(matrix);
trace("//rotate + scale + translate + matrix", result);
trace("");

trace("//middle application");
trace("");
trace("//double transform");

result = scale.clone();
result.concat(matrix);
result.concat(translate);
trace("//scale + matrix + translate", result);

result = translate.clone();
result.concat(matrix);
result.concat(scale);
trace("//translate + matrix + scale", result);

result = scale.clone();
result.concat(matrix);
result.concat(rotate);
trace("//scale + matrix + rotate", result);

result = rotate.clone();
result.concat(matrix);
result.concat(scale);
trace("//rotate + matrix + scale", result);

result = translate.clone();
result.concat(matrix);
result.concat(rotate);
trace("//translate + matrix + rotate", result);

result = rotate.clone();
result.concat(matrix);
result.concat(translate);
trace("//rotate + matrix + translate", result);
trace("");

trace("//triple transform #1");

result = scale.clone();
result.concat(matrix);
result.concat(translate);
result.concat(rotate);
trace("//scale + matrix + translate + rotate", result);

result = scale.clone();
result.concat(matrix);
result.concat(rotate);
result.concat(translate);
trace("//scale + matrix + rotate + translate", result);

result = translate.clone();
result.concat(matrix);
result.concat(scale);
result.concat(rotate);
trace("//translate + matrix + scale + rotate", result);

result = translate.clone();
result.concat(matrix);
result.concat(rotate);
result.concat(scale);
trace("//translate + matrix + rotate + scale", result);

result = rotate.clone();
result.concat(matrix);
result.concat(translate);
result.concat(scale);
trace("//rotate + matrix + translate + scale", result);

result = rotate.clone();
result.concat(matrix);
result.concat(scale);
result.concat(translate);
trace("//rotate + matrix + scale + translate", result);
trace("");

trace("//triple transform #2");

result = scale.clone();
result.concat(translate);
result.concat(matrix);
result.concat(rotate);
trace("//scale + translate + matrix + rotate", result);

result = scale.clone();
result.concat(rotate);
result.concat(matrix);
result.concat(translate);
trace("//scale + rotate + matrix + translate", result);

result = translate.clone();
result.concat(scale);
result.concat(matrix);
result.concat(rotate);
trace("//translate + scale + matrix + rotate", result);

result = translate.clone();
result.concat(rotate);
result.concat(matrix);
result.concat(scale);
trace("//translate + rotate + matrix + scale", result);

result = rotate.clone();
result.concat(translate);
result.concat(matrix);
result.concat(scale);
trace("//rotate + translate + matrix + scale", result);

result = rotate.clone();
result.concat(scale);
result.concat(matrix);
result.concat(translate);
trace("//rotate + scale + matrix + translate", result);
trace("");
trace("");

trace("/// invert");
trace("// matrix");
matrix = new Matrix();
trace(matrix);
trace("");

trace("// matrix.invert()");
matrix.invert();
trace(matrix);
trace("");
trace("");

trace("// matrix");
matrix = new Matrix(2, 3, 5, 7, 9, 11);
trace(matrix);
trace("");

trace("// matrix.invert()");
matrix.invert();
trace(matrix);
trace("");
trace("");

trace("/// createBox");
trace("// matrix = new Matrix();");
matrix = new Matrix();
trace(matrix);
trace("");

trace("// matrix.createBox(2, 3)");
matrix.createBox(2, 3);
trace(matrix);
trace("");

trace("// matrix.createBox(2, 3, 0)");
matrix.createBox(2, 3, 0);
trace(matrix);
trace("");

trace("// matrix.createBox(2, 3, 5)");
matrix.createBox(2, 3, 5);
trace(matrix);
trace("");

trace("// matrix.createBox(2, 3, 5, 7)");
matrix.createBox(2, 3, 5, 7);
trace(matrix);
trace("");

trace("// matrix.createBox(2, 3, 5, 7, 9)");
matrix.createBox(2, 3, 5, 7, 9);
trace(matrix);
trace("");
trace("");

trace("/// createGradientBox");
trace("// matrix = new Matrix();");
matrix = new Matrix();
trace(matrix);
trace("");

trace("// matrix.createGradientBox(200, 300)");
matrix.createGradientBox(200, 300);
trace(matrix);
trace("");

trace("// matrix.createGradientBox(200, 300, 0)");
matrix.createGradientBox(200, 300, 0);
trace(matrix);
trace("");

trace("// matrix.createGradientBox(200, 300, 500)");
matrix.createGradientBox(200, 300, 500);
trace(matrix);
trace("");

trace("// matrix.createGradientBox(200, 300, 500, 700)");
matrix.createGradientBox(200, 300, 500, 700);
trace(matrix);
trace("");

trace("// matrix.createGradientBox(200, 300, 500, 700, 900)");
matrix.createGradientBox(200, 300, 500, 700, 900);
trace(matrix);
trace("");
trace("");

trace("/// transformPoint");
trace("// matrix = new Matrix(2, 3, 5, 7, 11, 13);");
matrix = new Matrix(2, 3, 5, 7, 11, 13);
trace(matrix);
trace("");
trace("// matrix.transformPoint(new Point(1, 1));");
trace(matrix.transformPoint(new Point(1, 1)));
trace("");
trace("");

trace("/// deltaTransformPoint");
trace("// matrix = new Matrix(2, 3, 5, 7, 11, 13);");
matrix = new Matrix(2, 3, 5, 7, 11, 13);
trace(matrix);
trace("");
trace("// matrix.deltaTransformPoint(new Point(1, 1));");
trace(matrix.deltaTransformPoint(new Point(1, 1)));
trace("");
trace("");

trace("/// copyFrom");
trace("// matrix = new Matrix(2, 3, 5, 7, 11, 13);");
matrix = new Matrix(2, 3, 5, 7, 11, 13);
trace(matrix);
trace("// matrix2 = new Matrix();");
var matrix2 = new Matrix();
trace(matrix2);
trace("");
trace("// matrix2.copyFrom(matrix);");
matrix2.copyFrom(matrix);
trace(matrix2);
trace("");
trace("");

trace("/// setTo");
trace("// matrix = new Matrix();");
matrix = new Matrix();
trace(matrix);
trace("");
trace("// matrix.setTo(2, 3, 5, 7, 11, 13);");
matrix.setTo(2, 3, 5, 7, 11, 13);
trace(matrix);
trace("");
trace("");

trace("/// copyRowTo");
trace("// matrix = new Matrix(2, 3, 5, 7, 11, 13);");
matrix = new Matrix(2, 3, 5, 7, 11, 13);
trace("// vector = new Vector3D(1,2,3,4);");
var vector = new Vector3D(1,2,3,4);
trace(matrix);
trace(vector);
trace("");
trace("// matrix.copyRowTo(0, vector);");
matrix.copyRowTo(0, vector);
trace(vector);
trace("// matrix.copyRowTo(1, vector);");
matrix.copyRowTo(1, vector);
trace(vector);
trace("// matrix.copyRowTo(2, vector);");
matrix.copyRowTo(2, vector);
trace(vector);
trace("// matrix.copyRowTo(3, vector);");
matrix.copyRowTo(3, vector);
trace(vector);
trace("");
trace("");

trace("/// copyColumnTo");
trace("// matrix = new Matrix(2, 3, 5, 7, 11, 13);");
matrix = new Matrix(2, 3, 5, 7, 11, 13);
trace("// vector = new Vector3D(1,2,3,4);");
var vector = new Vector3D(1,2,3,4);
trace(matrix);
trace(vector);
trace("");
trace("// matrix.copyColumnTo(0, vector);");
matrix.copyColumnTo(0, vector);
trace(vector);
trace("// matrix.copyColumnTo(1, vector);");
matrix.copyColumnTo(1, vector);
trace(vector);
trace("// matrix.copyColumnTo(2, vector);");
matrix.copyColumnTo(2, vector);
trace(vector);
trace("// matrix.copyColumnTo(3, vector);");
matrix.copyColumnTo(3, vector);
trace(vector);
trace("");
trace("");

trace("/// copyRowFrom");
trace("// matrix = new Matrix(2, 3, 5, 7, 11, 13);");
matrix = new Matrix(2, 3, 5, 7, 11, 13);
trace("// vector = new Vector3D(17,19,23,29);");
var vector = new Vector3D(17,19,23,29);
trace(matrix);
trace(vector);
trace("");
trace("// matrix.copyRowFrom(0, vector);");
matrix.copyRowFrom(0, vector);
trace(matrix);
trace("// matrix.copyRowFrom(1, vector);");
matrix.copyRowFrom(1, vector);
trace(matrix);
trace("// matrix.copyRowFrom(2, vector);");
matrix.copyRowFrom(2, vector);
trace(matrix);
trace("// matrix.copyRowFrom(3, vector);");
matrix.copyRowFrom(3, vector);
trace(matrix);
trace("");
trace("");

trace("/// copyColumnFrom");
trace("// matrix = new Matrix(2, 3, 5, 7, 11, 13);");
matrix = new Matrix(2, 3, 5, 7, 11, 13);
trace("// vector = new Vector3D(17,19,23,29);");
var vector = new Vector3D(17,19,23,29);
trace(matrix);
trace(vector);
trace("");
trace("// matrix.copyColumnFrom(0, vector);");
matrix.copyColumnFrom(0, vector);
trace(matrix);
trace("// matrix.copyColumnFrom(1, vector);");
matrix.copyColumnFrom(1, vector);
trace(matrix);
trace("// matrix.copyColumnFrom(2, vector);");
matrix.copyColumnFrom(2, vector);
trace(matrix);
trace("// matrix.copyColumnFrom(3, vector);");
matrix.copyColumnFrom(3, vector);
trace(matrix);
trace("");
trace("");
