<template>
<div :id="uuid">
</div>
</template>
<script>
import JSONEditor from 'jsoneditor';
import 'jsoneditor/dist/jsoneditor.css';
import utils from '../tea/utils';

export default {
  props: {
    json: {
      type: Object,
      required: true,
    },
    onChange: {
      type: Function,
      required: true
    },
    mode: {
      type: String
    }
  },
  data(){
    return {
      uuid: utils.uuid(),
    }
  },
  created(){
    this.editor = null;
  },
  mounted(){
    const el = document.getElementById(this.uuid);
    const config = {
      mode: this.mode || 'code',
    };
    if(config.mode === 'code'){
      config.onChangeText = (text)=>{
        try{
          const json = JSON.parse(text);
          this.onChange(JSON.parse(text));
        }catch(e){
          alert('invalid json formatter');
        }
      };
    }
    else{
      config.onChangeJSON = (val)=>{
        this.onChange(val)
      };
    }
    this.editor = new JSONEditor(el, config, this.json);
  }
}
</script>