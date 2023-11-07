// GPTContentProcessor.ts

export class GPTContentProcessor {
  private buffer = '';

  private extractJsonObjects(buffer: string): string[] {
    const jsonObjects = [];
    let depth = 0;
    let startIndex = -1;

    // Iterate over the characters in the buffer to split complete JSON objects
    for (let i = 0; i < buffer.length; i++) {
      const char = buffer[i];
      if (char === '{') {
        // We have found the start of a JSON object
        if (depth === 0) startIndex = i;
        depth++;
      } else if (char === '}') {
        depth--;
        // We have found the end of a JSON object
        if (depth === 0 && startIndex !== -1) {
          // Extract the complete JSON object and add it to the list
          jsonObjects.push(buffer.substring(startIndex, i + 1));
          startIndex = -1;
        }
      }
    }

    return jsonObjects;
  }

  public processChunk(chunk: string): string {
    let content = '';

    // Add the new chunk to the buffer
    this.buffer += chunk;

    // Attempt to extract complete JSON objects from the buffer
    const jsonObjects = this.extractJsonObjects(this.buffer);

    jsonObjects.forEach(jsonObject => {
      try {
        // Parse the JSON object
        const data = JSON.parse(jsonObject);

        // Check if 'choices' and 'delta' fields exist and if 'content' is a string
        if (data.choices?.[0]?.delta?.content) {
          // If 'content' exists, append it to the output content string
          content += data.choices[0].delta.content;
        }
      } catch (error) {
        console.error('Failed to parse JSON object', error);
        // If an error occurred, this JSON object is likely corrupted or incomplete
      }
    });

    // Update the buffer with any remaining incomplete JSON
    this.buffer = this.buffer.substring(this.buffer.lastIndexOf('}') + 1);

    // Return any accumulated content from successfully parsed JSON objects
    return content;
  }
}