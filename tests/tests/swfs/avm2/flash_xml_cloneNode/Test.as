package {
  import flash.display.Sprite;
  public class Test extends Sprite { }
}

import flash.xml.XMLDocument;

var doc = new XMLDocument('<parent><child1 foo="bar"><amy>Amy</amy>Blub<bob>Bob <sponge/></bob></child1><child2/></parent>');

var child = doc.firstChild.firstChild;
trace("child.toString(): " + child.toString());
trace("child.firstChild: " + child.firstChild);
trace("child.lastChild: " + child.lastChild);
trace("child.previousSibling: " + child.previousSibling);
trace("child.nextSibling: " + child.nextSibling);
trace("child.parentNode: " + child.parentNode);

var clone = child.cloneNode(false);
trace("clone.toString(): " + clone.toString());
trace("clone.firstChild: " + clone.firstChild);
trace("clone.lastChild: " + clone.lastChild);
trace("clone.previousSibling: " + clone.previousSibling);
trace("clone.nextSibling: " + clone.nextSibling);
trace("clone.parentNode: " + clone.parentNode);

var deepClone = child.cloneNode(true);
trace("deepClone.toString(): " + deepClone.toString());
trace("deepClone.firstChild: " + deepClone.firstChild);
trace("deepClone.firstChild.parentNode: " + deepClone.firstChild.parentNode);
trace("deepClone.firstChild.parentNode === deepClone: " + (deepClone.firstChild.parentNode === deepClone));
trace("deepClone.firstChild.previousSibling: " + deepClone.firstChild.previousSibling);
trace("deepClone.firstChild.nextSibling: " + deepClone.firstChild.nextSibling);
trace("deepClone.lastChild: " + deepClone.lastChild);
trace("deepClone.previousSibling: " + deepClone.previousSibling);
trace("deepClone.nextSibling: " + deepClone.nextSibling);
trace("deepClone.parentNode: " + deepClone.parentNode);
