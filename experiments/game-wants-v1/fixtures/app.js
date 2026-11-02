// Authored deterministic evidence fixture ONLY; not an engine-created game.
const separate=document.body.dataset.fixture==='separate';
let cross,head,body,foods,score,crossEnd,growthEnd;
const view=document.querySelector('output');
function render(){view.textContent=`Crossing position ${cross.x},${cross.y}; crossing ${cross.y===0?'reached':'open'}; body ${body.length}; growth ${score}; head ${head.x},${head.y}; cross outcome ${crossEnd||'playing'}; growth outcome ${growthEnd||'playing'}`;}
function reset(){cross={x:0,y:4};head=separate?{x:0,y:4}:cross;body=[{...head}];foods=[{x:1,y:3},{x:2,y:3}];score=0;crossEnd='';growthEnd='';render();}
function step(delta,isGrowth){if((isGrowth?growthEnd:crossEnd)||(!separate&&(crossEnd||growthEnd)))return;const finish=reason=>{if(isGrowth)growthEnd=reason;else crossEnd=reason;render();};const origin=isGrowth?head:cross;const next={x:origin.x+delta[0],y:origin.y+delta[1]};if(next.x<0||next.x>3||next.y<0||next.y>4){finish('edge');return;}
 if((!separate||!isGrowth)&&next.x===3&&next.y===2){finish('hazard');return;}
 if(isGrowth){const eat=foods.some(f=>f.x===next.x&&f.y===next.y);const occupied=eat?body:body.slice(0,-1);if(occupied.some(p=>p.x===next.x&&p.y===next.y)){finish('body collision');return;}body.unshift({...next});if(eat){score++;foods=foods.filter(f=>f.x!==next.x||f.y!==next.y);}else body.pop();head=next;if(!separate)cross=head;}
 else cross=next;render();}
const arrows={ArrowUp:[0,-1],ArrowDown:[0,1],ArrowLeft:[-1,0],ArrowRight:[1,0]},wasd={w:[0,-1],s:[0,1],a:[-1,0],d:[1,0]};
document.querySelector('button').onclick=reset;
document.addEventListener('keydown',e=>{if(arrows[e.key]){e.preventDefault();step(arrows[e.key],!separate);}else if(separate&&wasd[e.key])step(wasd[e.key],true);});
reset();
