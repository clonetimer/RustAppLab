'use strict';
const $=id=>document.getElementById(id), status=$('status');
async function api(path,data) {
  const options=data===undefined?{}:{method:'POST',headers:{'content-type':'application/json','X-RustAppLab-Client':'1'},body:JSON.stringify(data)};
  if(data===null){options.method='POST';options.headers={'X-RustAppLab-Client':'1'};delete options.body;}
  const response=await fetch(path,options);
  const payload=await response.json();
  if(!response.ok) throw Error(payload.error||'HTTP '+response.status);
  return payload;
}
async function refresh(){
  const data=await api('/api/tickets');
  $('count').textContent='('+data.count+')';
  const list=$('tickets');list.replaceChildren();
  for(const ticket of data.tickets){
    const row=document.createElement('li');if(ticket.done)row.className='closed';
    const label=document.createElement('span');label.textContent='#'+ticket.id+' · '+ticket.title+(ticket.done?'（已关闭）':'');
    row.append(label);
    if(!ticket.done){
      const btn=document.createElement('button');btn.textContent='关闭';btn.addEventListener('click',async()=>{
        btn.disabled=true;
        try{await api('/api/tickets/'+ticket.id+'/close',null);status.textContent='已关闭工单 #'+ticket.id;await refresh();}
        catch(e){status.textContent='操作失败：'+e.message;btn.disabled=false;}
      });row.append(btn);
    }
    list.append(row);
  }
}
$('new').addEventListener('submit',async event=>{
  event.preventDefault();const button=event.currentTarget.querySelector('button');button.disabled=true;
  try{await api('/api/tickets',{title:$('title').value});$('title').value='';status.textContent='工单已保存';await refresh();}
  catch(e){status.textContent='保存失败：'+e.message;}finally{button.disabled=false;}
});
refresh().catch(e=>{status.textContent='读取失败：'+e.message;});
