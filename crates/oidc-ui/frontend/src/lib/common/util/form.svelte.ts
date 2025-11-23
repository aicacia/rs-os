import * as v from 'valibot';
import { ok, err, type Result } from '@aicacia/trycatch';

export type FieldState = 'validating' | 'valid' | 'invalid' | 'unset' | 'set';

export type BaseField<V extends v.BaseSchema<unknown, unknown, v.BaseIssue<unknown>>> = {
	value: v.InferOutput<V> | undefined;
	errors: v.InferIssue<V>[];
	state: FieldState;
	validate: () => Promise<Result<v.InferOutput<V>, v.ValiError<V>>>;
	reset: () => void;
};

export type PrimitiveField<V extends v.BaseSchema<unknown, unknown, v.BaseIssue<unknown>>> =
	BaseField<V>;

export type ArrayField<
	V extends v.ArraySchema<
		v.BaseSchema<unknown, unknown, v.BaseIssue<unknown>>,
		v.ErrorMessage<v.ArrayIssue> | undefined
	>
> = BaseField<V> & {
	items: BaseField<V['item']>[];
};

export type ObjectField<
	V extends v.ObjectSchema<v.ObjectEntries, v.ErrorMessage<v.ObjectIssue> | undefined>
> = BaseField<V> & {
	fields: { [K in keyof V['entries']]: BaseField<V['entries'][K]> };
};

export type Field<V extends v.BaseSchema<unknown, unknown, v.BaseIssue<unknown>>> =
	V extends v.ArraySchema<
		v.BaseSchema<unknown, unknown, v.BaseIssue<unknown>>,
		v.ErrorMessage<v.ArrayIssue> | undefined
	>
		? ArrayField<V>
		: V extends v.ObjectSchema<v.ObjectEntries, v.ErrorMessage<v.ObjectIssue> | undefined>
			? ObjectField<V>
			: PrimitiveField<V>;

export function createPrimitiveField<
	V extends v.BaseSchema<unknown, unknown, v.BaseIssue<unknown>>
>(schema: V, intialValue?: v.InferOutput<V>): PrimitiveField<V> {
	let value = $state<v.InferOutput<V>>(intialValue);
	let error = $state<v.ValiError<V>>();
	let state = $state<FieldState>('unset');

	async function validate(): Promise<Result<v.InferOutput<V>, v.ValiError<V>>> {
		state = 'validating';
		try {
			value = await v.parseAsync(schema, value);
			state = 'valid';
			return ok(value);
		} catch (e) {
			state = 'invalid';
			error = e as v.ValiError<V>;
			return err(error);
		}
	}

	function reset() {
		state = 'unset';
		value = intialValue;
	}

	return {
		get value() {
			return value;
		},
		set value(newValue: v.InferOutput<V> | undefined) {
			state = 'set';
			value = newValue;
		},
		get errors() {
			return error?.issues ?? [];
		},
		get state() {
			return state;
		},
		validate,
		reset
	};
}

export function createArrayField<
	V extends v.ArraySchema<
		v.BaseSchema<unknown, unknown, v.BaseIssue<unknown>>,
		v.ErrorMessage<v.ArrayIssue> | undefined
	>
>(schema: V, intialValue: v.InferOutput<V> = []): ArrayField<V> {
	const itemSchema = schema.item;
	const items: Field<V['item']>[] = intialValue.map(
		(itemValue) => createField(itemSchema, itemValue) as Field<V['item']>
	);
	let arrayValue = $state<v.InferOutput<V>>(intialValue);
	let arrayError = $state<v.ValiError<V>>();
	let arrayState = $state<FieldState>('unset');

	async function validate(): Promise<Result<v.InferOutput<V>, v.ValiError<V>>> {
		arrayState = 'validating';

		const output: v.InferOutput<V> = [];
		const issues: v.InferIssue<V>[] = [];

		await Promise.all(
			items.map(async (item, index) => {
				const [value, error] = await item.validate();

				if (error) {
					for (const issue of error.issues) {
						if (issue.path) {
							issue.path.unshift(index as never);
						}
						issues.push(issue);
					}
				} else {
					output.push(value);
					arrayValue[index] = value;
				}
			})
		);

		if (issues.length > 0) {
			const issueError = new v.ValiError(issues as [v.InferIssue<V>, ...v.InferIssue<V>[]]);
			arrayError = issueError;
			arrayState = 'invalid';
			return err(issueError);
		}

		arrayValue = output;
		arrayError = undefined;
		arrayState = 'valid';
		return ok(output);
	}

	function reset() {
		for (const item of items) {
			item.reset();
		}
		arrayValue = intialValue;
		arrayError = undefined;
		arrayState = 'unset';
	}

	return {
		get value() {
			return arrayValue;
		},
		set value(newValue: v.InferOutput<V>) {
			arrayState = 'set';
			arrayValue = newValue;
		},
		get errors() {
			return arrayError?.issues ?? [];
		},
		get state() {
			return arrayState;
		},
		items,
		validate,
		reset
	};
}

export function createObjectField<
	V extends v.ObjectSchema<v.ObjectEntries, v.ErrorMessage<v.ObjectIssue> | undefined>
>(schema: V, intialValue: v.InferOutput<V> = {}): ObjectField<V> {
	const fieldNames = Object.keys(schema.entries) as (keyof v.InferOutput<V>)[];
	const fields = {} as { [K in keyof V['entries']]: Field<V['entries'][K]> };

	for (const fieldName of fieldNames) {
		const entryName = fieldName as keyof V['entries'];
		const entry = schema.entries[entryName as keyof v.ObjectEntries];
		const entryIntialValue = intialValue[fieldName];

		fields[fieldName] = createField(entry, entryIntialValue) as never;
	}

	let objectValue = $state<v.InferOutput<V>>(intialValue);
	let objectError = $state<v.ValiError<V>>();
	let objectState = $state<FieldState>('unset');

	async function validate(): Promise<Result<v.InferOutput<V>, v.ValiError<V>>> {
		objectState = 'validating';

		const output = {} as v.InferOutput<V>;
		const issues: v.InferIssue<V>[] = [];

		await Promise.all(
			fieldNames.map(async (fieldName) => {
				const field = fields[fieldName];

				const [value, error] = await field.validate();

				if (error) {
					for (const issue of error.issues) {
						if (issue.path) {
							issue.path.unshift(fieldName as never);
						}
						issues.push(issue);
					}
				} else {
					output[fieldName] = value as never;
					objectValue[fieldName] = value as never;
				}
			})
		);
		if (issues.length > 0) {
			const issueError = new v.ValiError(issues as [v.InferIssue<V>, ...v.InferIssue<V>[]]);
			objectError = issueError;
			objectState = 'invalid';
			return err(issueError);
		}
		objectValue = output;
		objectError = undefined;
		objectState = 'valid';
		return ok(output);
	}

	function reset() {
		for (const fieldName of fieldNames) {
			fields[fieldName].reset();
		}
		objectValue = intialValue;
		objectError = undefined;
		objectState = 'unset';
	}

	return {
		get value() {
			return objectValue;
		},
		set value(newValue: v.InferOutput<V>) {
			objectState = 'set';
			objectValue = newValue;
		},
		get errors() {
			return objectError?.issues ?? [];
		},
		get state() {
			return objectState;
		},
		fields,
		validate,
		reset
	};
}

export function createField<V extends v.BaseSchema<unknown, unknown, v.BaseIssue<unknown>>>(
	schema: V,
	intialValue?: v.InferOutput<V>
): Field<V> {
	if ('item' in schema) {
		return createArrayField(schema as never, intialValue as never) as never;
	} else if ('entries' in schema) {
		return createObjectField(schema as never, intialValue as never) as never;
	} else {
		return createPrimitiveField(schema, intialValue) as never;
	}
}

export function createForm<
	V extends v.ObjectSchema<v.ObjectEntries, v.ErrorMessage<v.ObjectIssue> | undefined>
>(schema: V, intialValue: v.InferOutput<V> = {}) {
	return createObjectField(schema, intialValue);
}
