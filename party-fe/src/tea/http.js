import {_, axios} from 'tearust_utils';
import utils from './utils';

let _axios = null;
const init = ()=>{
  _axios = null;
  const baseUrl = utils.getHttpBaseUrl();
  _axios = axios.create({
    baseURL: baseUrl
  
  });

  _axios.interceptors.response.use((res)=>{
    console.log('[http response]', res.data);
    if(res.data){
      if(res.data.data){
        try{
          return Promise.resolve(JSON.parse(res.data.data));
        }catch(e){
          return res.data.data;
        }
      }
      return Promise.resolve(res.data);
    }
  }, (error)=>{
    return Promise.reject(error);
  });
}

const F = {
  initBaseUrl: init,
  requestActiveNodes() {
    return _axios.get('/api/request_active_nodes');
    // return [
    //   {
    //     "tea_id": "c7e016fad0796bb68594e49a6ef1942cf7e73497e69edb32d19ba2fab3696596",
    //     "nkn_id": "nkn_id",
    //     "http": "http://127.0.0.1:8000",
    //     "rsa": "LS0tLS1CRUdJTiBQVUJMSUMgS0VZLS0tLS0NCk1Gd3dEUVlKS29aSWh2Y05BUUVCQlFBRFN3QXdTQUpCQUxpV0pYYkxwYXlLL0hmQXFVRnVCOEUvdCtEQlFQUkgNCmFpQWRleFF6ODludThXSlJJUDc2QUJWdHdOeHN3WTNKZnZTVTMrcEkzaUhRem9LWEp0WTYxaVVDQXdFQUFRPT0NCi0tLS0tRU5EIFBVQkxJQyBLRVktLS0tLQ0K",
    //     "ping": "",
    //     "ws": "ws://127.0.0.1:8001",
    //     "credit": 0,
    //     "update_time": ""
    //   },
    // ]
  },
  putToIpfs(data) {
    return _axios.post('/ipfs-upload', data);
  },
  getFromIpfs(cid) {
    return _axios.get(`/ipfs-download?cid=${cid}`);
  },
  registerNewTask(proto_buf){
    return _axios.post('/api/register_task', proto_buf);
  },


  registerData(proto_buf_b64){
    return _axios.post('/api/register_data', proto_buf_b64);
  },

  /**
   * @param type: description, data
   * @param ekey1 
   * @param rsa_pub_key 
   */
  postDataWithRsaKey(type, data, ekey1, rsa_pub_key){
    rsa_pub_key = encodeURIComponent(rsa_pub_key);
    ekey1 = encodeURIComponent(ekey1);
    const url = `/ipfs?cid_type=${type}&ekey=${ekey1}&rsa_pub=${rsa_pub_key}`;

    return _axios.post(url, data);
  },

  requestBeMyDelegate(proto_buf_b64){
    return _axios.post('/api/be_my_delegate', proto_buf_b64);
  },

  repinDeployment(proto_buf_b64) {
    return _axios.post('/api/repin_deployment', proto_buf_b64);
  },

  requestErrandTask(url, json_b64){
    return _axios.post(url, json_b64);
  },
  getBalanceInfo(proto_buf_b64){
    return _axios.post('/api/get_balance_info', proto_buf_b64);
  },

  post(url, data){
    return _axios.post(url, data);
  },


  query_deployment_id_by_session_id(seesion_id){
    return _axios.post('/api/query_deployment_id_by_session_id', seesion_id);
  },
  start_query_pinners_by_deployment_id(deployment_id){
    return _axios.post('/api/start_query_pinners_by_deployment_id', deployment_id);
  },
  query_pinners_by_deployment_id(deployment_id){
    return _axios.post('/api/query_pinners_by_deployment_id', deployment_id);
  }
};


export default F;