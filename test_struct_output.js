
function Point(init = {}) {
  return {
    x: (init.x !== undefined ? init.x : undefined),
    y: (init.y !== undefined ? init.y : undefined)
  };
}
let p = Point({x: 5, y: 0});
console.log(p.x);
console.log(p.y);



