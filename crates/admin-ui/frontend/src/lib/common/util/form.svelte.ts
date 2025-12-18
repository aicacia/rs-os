import * as v from 'valibot';
import { debounce } from '@aicacia/debounce';
import { ok, err, type Result } from '@aicacia/trycatch';

export type FieldState = 'validating' | 'valid' | 'invalid' | 'unset' | 'set';

export interface CommonField<V extends v.BaseSchema<unknown, unknown, v.BaseIssue<unknown>>> {
	reset(newInitialValue?: v.InferInput<V>): void;
	validate(): Promise<Result<v.InferOutput<V>, v.ValiError<V>>>;
}

export interface PrimitiveField<
	V extends v.BaseSchema<unknown, unknown, v.BaseIssue<unknown>>
> extends CommonField<V> {
	value: v.InferInput<V> | undefined;
	issues: v.InferIssue<V>[];
}

export interface ArrayField<
	V extends v.ArraySchema<
		v.BaseSchema<unknown, unknown, v.BaseIssue<unknown>>,
		v.ErrorMessage<v.ArrayIssue> | undefined
	>
> extends CommonField<V> {
	items: Field<V['item']>[];
}

export interface ObjectField<
	V extends v.ObjectSchema<v.ObjectEntries, v.ErrorMessage<v.ObjectIssue> | undefined>
> extends CommonField<V> {
	fields: { [K in keyof V['entries']]: Field<V['entries'][K]> };
}

export type Field<V extends v.BaseSchema<unknown, unknown, v.BaseIssue<unknown>>> =
	V extends v.ArraySchema<
		v.BaseSchema<unknown, unknown, v.BaseIssue<unknown>>,
		v.ErrorMessage<v.ArrayIssue> | undefined
	>
		? ArrayField<V>
		: V extends v.ObjectSchema<v.ObjectEntries, v.ErrorMessage<v.ObjectIssue> | undefined>
			? ObjectField<V>
			: PrimitiveField<V>;

export function createObjectField<
	V extends v.ObjectSchema<v.ObjectEntries, v.ErrorMessage<v.ObjectIssue> | undefined>
>(schema: V, initialValue: v.InferInput<V> = {}): ObjectField<V> {
	const objectField: ObjectField<V> = {
		fields: {} as { [K in keyof V['entries']]: Field<V['entries'][K]> },
		validate,
		reset
	};

	for (const [fieldName, fieldSchema] of Object.entries(schema.entries) as [
		keyof V['entries'],
		V['entries'][keyof V['entries']]
	][]) {
		objectField.fields[fieldName] = createField(
			fieldSchema,
			initialValue[fieldName as keyof v.InferInput<V>]
		) as Field<V['entries'][typeof fieldName]>;
	}

	async function validate(): Promise<Result<v.InferOutput<V>, v.ValiError<V>>> {
		const output = {} as v.InferOutput<V>;
		const issues: v.InferIssue<V>[] = [];

		await Promise.all(
			Object.entries(objectField.fields).map(async ([fieldName, field]) => {
				const [fieldOutput, fieldError] = await field.validate();

				if (fieldError) {
					for (const fieldIssue of fieldError.issues) {
						if (fieldIssue.path) {
							fieldIssue.path.unshift(fieldName);
						}
						issues.push(fieldIssue as v.InferIssue<V>);
					}
				} else {
					output[fieldName as keyof v.InferOutput<V>] = fieldOutput;
				}
			})
		);

		if (issues.length > 0) {
			const issueError = new v.ValiError(issues as [v.InferIssue<V>, ...v.InferIssue<V>[]]);
			return err(issueError);
		}

		return ok(output);
	}

	function reset(newInitialValue?: v.InferInput<V>) {
		for (const [fieldName, field] of Object.entries(objectField.fields)) {
			field.reset(
				newInitialValue?.[fieldName as keyof v.InferInput<V>] ??
					initialValue[fieldName as keyof v.InferInput<V>]
			);
		}
	}

	return objectField;
}

export function createArrayField<
	V extends v.ArraySchema<
		v.BaseSchema<unknown, unknown, v.BaseIssue<unknown>>,
		v.ErrorMessage<v.ArrayIssue> | undefined
	>
>(schema: V, initialValue: v.InferInput<V> = []): ArrayField<V> {
	const arrayField: ArrayField<V> = {
		items: initialValue.map((itemValue) => createField(schema.item, itemValue)) as Field<
			V['item']
		>[],
		validate,
		reset
	};

	async function validate(): Promise<Result<v.InferOutput<V>, v.ValiError<V>>> {
		const output: v.InferOutput<V> = [] as unknown as v.InferOutput<V>;
		const issues: v.InferIssue<V>[] = [];

		await Promise.all(
			arrayField.items.map(async (itemField, index) => {
				const [itemOutput, itemError] = await itemField.validate();

				if (itemError) {
					for (const itemIssue of itemError.issues) {
						if (itemIssue.path) {
							itemIssue.path.unshift(index as never);
						}
						issues.push(itemIssue as v.InferIssue<V>);
					}
				} else {
					(output as unknown as v.InferOutput<V>)[index] = itemOutput;
				}
			})
		);

		if (issues.length > 0) {
			const issueError = new v.ValiError(issues as [v.InferIssue<V>, ...v.InferIssue<V>[]]);
			return err(issueError);
		}

		return ok(output);
	}

	function reset(newInitialValue?: v.InferInput<V>) {
		for (let i = 0; i < arrayField.items.length; i++) {
			arrayField.items[i].reset((newInitialValue?.[i] as never) ?? (initialValue[i] as never));
		}
	}

	return arrayField;
}

export function createPrimitiveField<
	V extends v.BaseSchema<unknown, unknown, v.BaseIssue<unknown>>
>(schema: V, initialValue: v.InferInput<V> = undefined): PrimitiveField<V> {
	let value = $state(initialValue);
	const issues = $state<v.InferIssue<V>[]>([]);

	async function validate(): Promise<Result<v.InferOutput<V>, v.ValiError<V>>> {
		try {
			const output = await v.parseAsync(schema, value);
			issues.length = 0;
			return ok(output);
		} catch (e) {
			issues.length = 0;

			if (e instanceof v.ValiError) {
				issues.push(...e.issues);
				return err(new v.ValiError<V>(e.issues));
			} else {
				throw e;
			}
		}
	}
	const debounceValidate = debounce(validate, 300);

	function reset(newInitialValue?: v.InferInput<V>) {
		value = newInitialValue ?? initialValue;
		issues.length = 0;
	}

	return {
		get value() {
			return value;
		},
		set value(newValue: v.InferInput<V> | undefined) {
			value = newValue;
			void debounceValidate();
		},
		get issues() {
			return issues;
		},
		validate,
		reset
	};
}

export function createField<V extends v.BaseSchema<unknown, unknown, v.BaseIssue<unknown>>>(
	schema: V,
	initialValue: v.InferInput<V> = undefined
): Field<V> {
	if (schema.type === 'object') {
		return createObjectField(
			schema as V & v.ObjectSchema<v.ObjectEntries, v.ErrorMessage<v.ObjectIssue> | undefined>,
			initialValue as v.InferInput<
				V & v.ObjectSchema<v.ObjectEntries, v.ErrorMessage<v.ObjectIssue> | undefined>
			>
		) as Field<V>;
	} else if (schema.type === 'array') {
		return createArrayField(
			schema as V &
				v.ArraySchema<
					v.BaseSchema<unknown, unknown, v.BaseIssue<unknown>>,
					v.ErrorMessage<v.ArrayIssue> | undefined
				>,
			initialValue as v.InferInput<
				V &
					v.ArraySchema<
						v.BaseSchema<unknown, unknown, v.BaseIssue<unknown>>,
						v.ErrorMessage<v.ArrayIssue> | undefined
					>
			>
		) as Field<V>;
	} else {
		return createPrimitiveField(schema, initialValue) as Field<V>;
	}
}

export function createForm<
	V extends v.ObjectSchema<v.ObjectEntries, v.ErrorMessage<v.ObjectIssue> | undefined>
>(schema: V, initialValue: v.InferInput<V> = {}) {
	const fields = $state({} as { [K in keyof V['entries']]: Field<V['entries'][K]> });

	for (const [fieldName, fieldSchema] of Object.entries(schema.entries) as [
		keyof V['entries'],
		V['entries'][keyof V['entries']]
	][]) {
		fields[fieldName] = createField(fieldSchema, initialValue[fieldName as keyof v.InferInput<V>]);
	}

	async function validate(): Promise<Result<v.InferOutput<V>, v.ValiError<V>>> {
		const output = {} as v.InferOutput<V>;
		const issues: v.InferIssue<V>[] = [];

		await Promise.all(
			Object.entries(fields).map(async ([fieldName, field]) => {
				const [fieldOutput, fieldError] = await field.validate();

				if (fieldError) {
					for (const fieldIssue of fieldError.issues) {
						if (fieldIssue.path) {
							fieldIssue.path.unshift(fieldName);
						}
						issues.push(fieldIssue as v.InferIssue<V>);
					}
				} else {
					output[fieldName as keyof v.InferOutput<V>] = fieldOutput;
				}
			})
		);

		if (issues.length > 0) {
			const issueError = new v.ValiError(issues as [v.InferIssue<V>, ...v.InferIssue<V>[]]);
			return err(issueError);
		}

		return ok(output);
	}

	function reset(newInitialValue?: v.InferInput<V>) {
		for (const fieldName of Object.keys(fields) as (keyof V['entries'])[]) {
			fields[fieldName].reset(
				(newInitialValue?.[fieldName as keyof v.InferInput<V>] ??
					initialValue[fieldName as keyof v.InferInput<V>]) as never
			);
		}
	}

	return {
		fields,
		validate,
		reset
	};
}
