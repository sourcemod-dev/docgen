import { Declaration } from './base';
import { Function } from './function';
import { IEnumStruct, IField } from '../../interfaces';

export class EnumStruct extends Declaration implements IEnumStruct {
    /**
     * @brief Functions within this enum struct
     * @readonly
     */
     readonly methods: Function[];

     /**
      * @brief Fields within this enum struct
      * @readonly
      */
     readonly fields: IField[];

     public constructor(es: IEnumStruct) {
        super(es);

        this.methods = es.methods.map(f => new Function(f));
        this.fields = es.fields;
     }
}
