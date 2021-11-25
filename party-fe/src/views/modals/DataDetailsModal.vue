<template>
  <el-dialog
    :title="title || 'Details'"
    :visible="visible"
    width="70%"
    :close-on-click-modal="false"
    custom-class="tea-modal"
    :destroy-on-close="true"
    @close="$store.commit('modal/close', 'data_details')"
  >

    <div class="tea-modal-card">
      
      <div class="x-list">

        <div v-for="(val, key) in param" :key="key" class="x-item">
          <b style="width: 400px;">{{key}}</b>
          <span :inner-html.prop="toVal(val)"></span>
        </div>

      </div>

    </div>

    <span slot="footer" class="dialog-footer">
      <el-button size="small" @click="$store.commit('modal/close', 'data_details')">Close</el-button>
    </span>

  </el-dialog>


</template>
<script>
import { mapState } from 'vuex';
import store from '../../store/index';
import utils from '../../tea/utils';
import {_} from 'tearust_utils';
import {hexToString} from 'tearust_layer1';
export default {
  data(){
    return {
      form: {
        price: null,
      }
    };
  },
  computed: {
    ...mapState('modal', {
      visible: state => store.state.modal.data_details.visible,
      param: state => store.state.modal.data_details.param,
      title: state => store.state.modal.data_details.title,
    })
  },

  methods: {
    toVal(val){
      if(_.isString(val) && _.startsWith(val, '0x')){
        return hexToString(val);
      }

      return val;
    }
  }
}
</script>