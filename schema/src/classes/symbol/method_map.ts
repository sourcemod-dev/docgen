import { IMethodMap, IProperty } from '../../interfaces';
import { Declaration } from './base';
import { Function } from './function';

export class MethodMap extends Declaration implements IMethodMap {
    /**
     * @brief Parent inheritance if any
     * @readonly
     */
     readonly parent?: string;

     /**
      * @brief Functions within this methodmap
      * @readonly
      */
     readonly methods: Function[];
 
     /**
      * @brief Properties within this methodmap
      * @readonly
      */
     readonly properties: IProperty[];

    public constructor(mm: IMethodMap) {
        super(mm);

        this.parent = mm.parent;
        this.methods = mm.methods.map(f => new Function(f));
        this.properties = mm.properties;
    }
}
