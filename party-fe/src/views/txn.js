import {_, axios, moment, uuid} from 'tearust_utils';
import utils from '../tea/utils';
import bbs from './bbs';

const F = {

  async txn_request(method, param){
    const _uuid = uuid();
    console.log("prepare for txn: ", method, _uuid);
    
    const _axios = bbs.getAxios();

    const txn_uuid = 'txn_'+_uuid;
    try{
      bbs.log("Send txn request...");
      console.log("Send txn request...");
      const step1_rs = await _axios.post('/tapp/'+method, {
        ...param,
        uuid: txn_uuid,
      });
      console.log("step_1 result: ", step1_rs);
    }catch(e){
      console.error("step_1 error: ", e);

      throw e;
    }

    bbs.log('Wait for query txn hash...');
    console.log('Wait for query txn hash...');
    await utils.sleep(5000);

    let step_2_rs = null;
    const step_2_loop = async ()=>{
      try{
        console.log('query result for '+txn_uuid+'...');
        step_2_rs = await _axios.post('/tapp/query_result', {
          uuid: txn_uuid,
        });

        step_2_rs = utils.parseJSON(step_2_rs);
      }catch(e){
        console.log("step2 error: ", e);
        // rs = e.message;
        step_2_rs = null;
        await utils.sleep(3000);
        await step_2_loop();
      }
  
    };
  
    bbs.log("Start to query txn result...");
    console.log("Start to query txn result...");
    await step_2_loop();

    console.log("step2 result: ", step_2_rs);

    bbs.log('Wait for next step...');
    console.log('Wait for next step...');
    utils.sleep(5000);

    const step_3_hash = step_2_rs.hash;
    const hash_uuid = "hash_"+_uuid;
    let step_3_rs = null;
    let step_4_rs = null;
    let sn = 0;
    const step_4_loop = async ()=>{
      if(sn > 10) {
        step_4_rs = {
          'status': false,
          'error': 'request timeout',
        };
        return;
      }
      try{
        bbs.log("Send query txn hash request...");
        console.log('Send query txn hash request...');
        step_3_rs = await _axios.post('/tapp/queryHashResult', {
          hash: step_3_hash,
          uuid: hash_uuid,
        });
    
        bbs.log('Wait for query txn hash result...');
        console.log('Wait for query txn hash result...');
        await utils.sleep(5000);

        console.log('query hash result for '+hash_uuid+'...');
        step_4_rs = await _axios.post('/tapp/query_result', {
          uuid: hash_uuid,
        });

        step_4_rs = utils.parseJSON(step_4_rs);
        if(!step_4_rs.status) throw step_4_rs.error;
      }catch(e){
        console.log("step4 error: ", e);

        if(e !== 'wait'){
          throw e;
        }
        
        // rs = e.message;
        step_4_rs = null;
        sn++;
        await utils.sleep(5000);
        await step_4_loop();
      }
  
    };
  
    bbs.log("Start to query hash result...");
    console.log("Start to query hash result...");
    await step_4_loop();

    console.log("step4 result: ", step_4_rs);
    if(step_4_rs.error){
      throw step_4_rs.error;
    }

    if(!step_4_rs.need_query){
      return step_4_rs;
    }

    // continue query

    let step_5_rs = null;
    let step_5_uuid = step_4_rs.query_uuid || _uuid;
    let step_5_n = 0;
    const step_5_loop = async ()=>{
      if(step_5_n > 3){
        throw 'query timeout...';
      }
      try{
        console.log('continue query for '+step_5_uuid+'...');
        step_5_rs = await _axios.post('/tapp/query_result', {
          uuid: step_5_uuid,
        });

        step_5_rs = utils.parseJSON(step_5_rs);
      }catch(e){
        console.log("step5 error: ", e);
        step_5_n ++;
        step_5_rs = null;
        await utils.sleep(5000);
        await step_5_loop();
      }
    };

    bbs.log("Start to query action result...");
    console.log("Start to query action result...");
    await step_5_loop();
    console.log("step5 result: ", step_5_rs);

    const rs = step_5_rs;

    return rs;
  }

};


export default F;